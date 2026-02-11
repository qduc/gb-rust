#[derive(Clone)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub dac_enabled: bool,

    pub nr41: u8,
    pub nr42: u8,
    pub nr43: u8,
    pub nr44: u8,

    length_counter: u16,
    timer: u16,
    volume: u8,
    env_timer: u8,
    lfsr: u16,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            nr41: 0,
            nr42: 0,
            nr43: 0,
            nr44: 0,
            length_counter: 0,
            timer: 1,
            volume: 0,
            env_timer: 0,
            lfsr: 0x7FFF,
        }
    }

    pub fn powered_register_clear(&mut self) {
        self.enabled = false;
        self.dac_enabled = false;
        self.nr41 = 0;
        self.nr42 = 0;
        self.nr43 = 0;
        self.nr44 = 0;
        self.length_counter = 0;
        self.timer = 1;
        self.volume = 0;
        self.env_timer = 0;
        self.lfsr = 0x7FFF;
    }

    pub fn write_nr41(&mut self, value: u8) {
        self.nr41 = value;
        let len = 64 - u16::from(value & 0x3F);
        self.length_counter = if len == 0 { 64 } else { len };
    }

    pub fn write_nr42(&mut self, value: u8) {
        self.nr42 = value;
        self.dac_enabled = (value & 0xF8) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_nr43(&mut self, value: u8) {
        self.nr43 = value;
    }

    pub fn write_nr44(&mut self, value: u8) {
        self.nr44 = value & 0xC0;
    }

    pub fn trigger(&mut self) {
        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        self.timer = self.period();
        self.env_timer = self.envelope_period();
        self.volume = (self.nr42 >> 4) & 0x0F;
        self.lfsr = 0x7FFF;
        self.enabled = self.dac_enabled;
    }

    pub fn tick_timer(&mut self) {
        if self.timer > 1 {
            self.timer -= 1;
            return;
        }

        self.timer = self.period();

        let xor = (self.lfsr & 0x01) ^ ((self.lfsr >> 1) & 0x01);
        self.lfsr >>= 1;
        self.lfsr |= xor << 14;

        if (self.nr43 & 0x08) != 0 {
            self.lfsr &= !(1 << 6);
            self.lfsr |= xor << 6;
        }
    }

    pub fn clock_length(&mut self) {
        if (self.nr44 & 0x40) == 0 {
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
        let period = self.nr42 & 0x07;
        if period == 0 {
            return;
        }

        if self.env_timer > 1 {
            self.env_timer -= 1;
            return;
        }

        self.env_timer = self.envelope_period();
        let increase = (self.nr42 & 0x08) != 0;
        if increase {
            if self.volume < 15 {
                self.volume += 1;
            }
        } else if self.volume > 0 {
            self.volume -= 1;
        }
    }

    fn envelope_period(&self) -> u8 {
        let p = self.nr42 & 0x07;
        if p == 0 {
            8
        } else {
            p
        }
    }

    fn period(&self) -> u16 {
        const DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];
        let divisor = DIVISORS[(self.nr43 & 0x07) as usize];
        let shift = (self.nr43 >> 4) & 0x0F;
        divisor << shift
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let phase = if (self.lfsr & 0x01) == 0 { 1.0 } else { -1.0 };
        phase * (self.volume as f32 / 15.0)
    }

    pub fn length_counter(&self) -> u16 {
        self.length_counter
    }
}

impl Default for NoiseChannel {
    fn default() -> Self {
        Self::new()
    }
}
