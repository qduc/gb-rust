#[derive(Clone)]
pub struct WaveChannel {
    pub enabled: bool,
    pub dac_enabled: bool,

    pub nr30: u8,
    pub nr31: u8,
    pub nr32: u8,
    pub nr33: u8,
    pub nr34: u8,

    length_counter: u16,
    timer: u16,
    sample_index: u8,
    wave_ram: [u8; 16],
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            nr30: 0,
            nr31: 0,
            nr32: 0,
            nr33: 0,
            nr34: 0,
            length_counter: 0,
            timer: 1,
            sample_index: 0,
            wave_ram: [0; 16],
        }
    }

    pub fn powered_register_clear(&mut self) {
        self.enabled = false;
        self.dac_enabled = false;
        self.nr30 = 0;
        self.nr31 = 0;
        self.nr32 = 0;
        self.nr33 = 0;
        self.nr34 = 0;
        self.length_counter = 0;
        self.timer = 1;
        self.sample_index = 0;
    }

    pub fn read_wave_ram(&self, index: usize, cgb_mode: bool) -> u8 {
        // CGB wave RAM reads while the channel is enabled return the *currently playing*
        // wave byte, regardless of address (blargg `cgb_sound` behavior).
        if cgb_mode && self.enabled {
            self.wave_ram[self.current_wave_byte_index()]
        } else {
            self.wave_ram[index]
        }
    }

    pub fn write_wave_ram(&mut self, index: usize, value: u8, cgb_mode: bool) {
        // CGB wave RAM writes while the channel is enabled affect the *currently playing*
        // wave byte, regardless of address.
        if cgb_mode && self.enabled {
            let idx = self.current_wave_byte_index();
            self.wave_ram[idx] = value;
        } else {
            self.wave_ram[index] = value;
        }
    }

    pub fn write_nr30(&mut self, value: u8) {
        self.nr30 = value;
        self.dac_enabled = (value & 0x80) != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn write_nr31(&mut self, value: u8) {
        self.nr31 = value;
        let len = 256 - u16::from(value);
        self.length_counter = if len == 0 { 256 } else { len };
    }

    pub fn write_nr32(&mut self, value: u8) {
        self.nr32 = value;
    }

    pub fn write_nr33(&mut self, value: u8, cgb_mode: bool) {
        self.nr33 = value;
        let _ = cgb_mode;
    }

    pub fn write_nr34(&mut self, value: u8, cgb_mode: bool) {
        self.nr34 = value & 0xC7;
        let _ = cgb_mode;
    }

    pub fn trigger(&mut self) {
        if self.length_counter == 0 {
            self.length_counter = 256;
        }

        self.timer = self.period();
        self.sample_index = 0;
        self.enabled = self.dac_enabled;
    }

    pub fn tick_timer(&mut self) {
        if self.timer > 1 {
            self.timer -= 1;
            return;
        }

        self.timer = self.period();
        self.sample_index = (self.sample_index + 1) & 31;
    }

    pub fn clock_length(&mut self) {
        if (self.nr34 & 0x40) == 0 {
            return;
        }

        if self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    fn frequency(&self) -> u16 {
        ((u16::from(self.nr34) & 0x07) << 8) | u16::from(self.nr33)
    }

    fn period(&self) -> u16 {
        (2048 - self.frequency()) * 2
    }

    fn volume_shift(&self) -> Option<u8> {
        match (self.nr32 >> 5) & 0x03 {
            0 => None,
            1 => Some(0),
            2 => Some(1),
            3 => Some(2),
            _ => None,
        }
    }

    pub fn output(&self) -> f32 {
        if !self.enabled || !self.dac_enabled {
            return 0.0;
        }

        let byte = self.wave_ram[(self.sample_index / 2) as usize];
        let nibble = if (self.sample_index & 1) == 0 {
            byte >> 4
        } else {
            byte & 0x0F
        };

        let Some(shift) = self.volume_shift() else {
            // Volume code 0 is mute.
            return 0.0;
        };

        let sample = nibble >> shift;
        (sample as f32 / 7.5) - 1.0
    }

    pub fn length_counter(&self) -> u16 {
        self.length_counter
    }

    fn current_wave_byte_index(&self) -> usize {
        (self.sample_index / 2) as usize
    }
}

impl Default for WaveChannel {
    fn default() -> Self {
        Self::new()
    }
}
