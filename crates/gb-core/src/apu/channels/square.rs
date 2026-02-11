const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

#[derive(Clone)]
pub struct SquareChannel {
    pub enabled: bool,
    pub dac_enabled: bool,

    pub sweep: u8,       // NR10 (only used on CH1)
    pub duty_length: u8, // NRx1
    pub envelope: u8,    // NRx2
    pub freq_lo: u8,     // NRx3
    pub freq_hi: u8,     // NRx4

    length_counter: u16,
    timer: u16,
    duty_step: u8,
    volume: u8,
    env_timer: u8,

    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow_freq: u16,
    sweep_negate_used: bool,
    has_sweep: bool,
}

impl SquareChannel {
    pub fn new(has_sweep: bool) -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            sweep: 0,
            duty_length: 0,
            envelope: 0,
            freq_lo: 0,
            freq_hi: 0,
            length_counter: 0,
            timer: 1,
            duty_step: 0,
            volume: 0,
            env_timer: 0,
            sweep_timer: 0,
            sweep_enabled: false,
            sweep_shadow_freq: 0,
            sweep_negate_used: false,
            has_sweep,
        }
    }

    pub fn powered_register_clear(&mut self) {
        self.sweep = 0;
        self.duty_length = 0;
        self.envelope = 0;
        self.freq_lo = 0;
        self.freq_hi = 0;

        self.enabled = false;
        self.dac_enabled = false;
        self.length_counter = 0;
        self.timer = 1;
        self.duty_step = 0;
        self.volume = 0;
        self.env_timer = 0;
        self.sweep_timer = 0;
        self.sweep_enabled = false;
        self.sweep_shadow_freq = 0;
        self.sweep_negate_used = false;
    }

    pub fn write_sweep(&mut self, value: u8) {
        if !self.has_sweep {
            return;
        }

        let old_negate = (self.sweep & 0x08) != 0;
        let new_negate = (value & 0x08) != 0;
        if old_negate && !new_negate && self.sweep_negate_used {
            self.enabled = false;
        }

        self.sweep = value;
    }

    pub fn write_duty_length(&mut self, value: u8) {
        self.duty_length = value;
        let len = 64 - u16::from(value & 0x3F);
        self.length_counter = if len == 0 { 64 } else { len };
    }

    pub fn write_envelope(&mut self, value: u8) {
        self.envelope = value;
        self.dac_enabled = (value & 0xF8) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_freq_lo(&mut self, value: u8) {
        self.freq_lo = value;
    }

    pub fn write_freq_hi(&mut self, value: u8) {
        self.freq_hi = value & 0xC7;
    }

    pub fn trigger(&mut self) {
        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.timer = self.period();
        self.duty_step = 0;
        self.volume = (self.envelope >> 4) & 0x0F;
        self.env_timer = self.envelope_period();
        self.enabled = self.dac_enabled;

        if self.has_sweep {
            self.sweep_shadow_freq = self.frequency();
            self.sweep_timer = self.sweep_period();
            let shift = self.sweep & 0x07;
            let period = (self.sweep >> 4) & 0x07;
            self.sweep_enabled = period != 0 || shift != 0;
            self.sweep_negate_used = false;

            if shift != 0 {
                let _ = self.sweep_calculate();
            }
        }
    }

    pub fn frequency(&self) -> u16 {
        ((u16::from(self.freq_hi) & 0x07) << 8) | u16::from(self.freq_lo)
    }

    pub fn set_frequency(&mut self, freq: u16) {
        self.freq_lo = (freq & 0x00FF) as u8;
        self.freq_hi = (self.freq_hi & 0xF8) | ((freq >> 8) as u8 & 0x07);
    }

    fn period(&self) -> u16 {
        (2048 - self.frequency()) * 4
    }

    fn envelope_period(&self) -> u8 {
        let p = self.envelope & 0x07;
        if p == 0 {
            8
        } else {
            p
        }
    }

    fn sweep_period(&self) -> u8 {
        let p = (self.sweep >> 4) & 0x07;
        if p == 0 {
            8
        } else {
            p
        }
    }

    pub fn tick_timer(&mut self) {
        if self.timer > 1 {
            self.timer -= 1;
            return;
        }

        self.timer = self.period();
        self.duty_step = (self.duty_step + 1) & 0x07;
    }

    pub fn clock_length(&mut self) {
        if (self.freq_hi & 0x40) == 0 {
            return;
        }

        if self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn clock_envelope(&mut self) {
        let period = self.envelope & 0x07;
        if period == 0 {
            return;
        }

        if self.env_timer > 1 {
            self.env_timer -= 1;
            return;
        }

        self.env_timer = self.envelope_period();
        let increase = (self.envelope & 0x08) != 0;
        if increase {
            if self.volume < 15 {
                self.volume += 1;
            }
        } else if self.volume > 0 {
            self.volume -= 1;
        }
    }

    pub fn clock_sweep(&mut self) {
        if !self.has_sweep || !self.sweep_enabled {
            return;
        }

        if self.sweep_timer > 1 {
            self.sweep_timer -= 1;
            return;
        }

        self.sweep_timer = self.sweep_period();

        let period = (self.sweep >> 4) & 0x07;
        if period == 0 {
            return;
        }

        if let Some(new_freq) = self.sweep_calculate() {
            if (self.sweep & 0x07) != 0 {
                self.sweep_shadow_freq = new_freq;
                self.set_frequency(new_freq);
                let _ = self.sweep_calculate();
            }
        }
    }

    fn sweep_calculate(&mut self) -> Option<u16> {
        let shift = self.sweep & 0x07;
        let delta = self.sweep_shadow_freq >> shift;
        let negate = (self.sweep & 0x08) != 0;

        let new_freq = if negate {
            self.sweep_negate_used = true;
            self.sweep_shadow_freq.wrapping_sub(delta)
        } else {
            self.sweep_shadow_freq.wrapping_add(delta)
        };

        if new_freq > 2047 {
            self.enabled = false;
            None
        } else {
            Some(new_freq)
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let duty = (self.duty_length >> 6) as usize;
        let bit = DUTY_TABLE[duty][self.duty_step as usize];
        let phase = if bit == 0 { -1.0 } else { 1.0 };
        phase * (self.volume as f32 / 15.0)
    }

    pub fn length_counter(&self) -> u16 {
        self.length_counter
    }
}
