use crate::cartridge::mbc::Mbc;
use serde::{Deserialize, Serialize};

const CYCLES_PER_SECOND: u32 = 4_194_304;

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
struct Rtc {
    sec: u8,
    min: u8,
    hour: u8,
    day_low: u8,
    day_high: u8,
}

impl Rtc {
    fn read_reg(self, reg: u8) -> u8 {
        match reg {
            0x08 => self.sec,
            0x09 => self.min,
            0x0A => self.hour,
            0x0B => self.day_low,
            0x0C => self.day_high | 0x3E,
            _ => 0xFF,
        }
    }

    fn write_reg(&mut self, reg: u8, val: u8) {
        match reg {
            0x08 => self.sec = val % 60,
            0x09 => self.min = val % 60,
            0x0A => self.hour = val % 24,
            0x0B => self.day_low = val,
            0x0C => self.day_high = val & 0xC1,
            _ => {}
        }
    }

    fn halted(self) -> bool {
        (self.day_high & 0x40) != 0
    }

    fn day_counter(self) -> u16 {
        (((self.day_high & 0x01) as u16) << 8) | self.day_low as u16
    }

    fn set_day_counter(&mut self, day: u16) {
        self.day_low = (day & 0xFF) as u8;
        self.day_high = (self.day_high & 0xFE) | (((day >> 8) & 0x01) as u8);
    }

    fn increment_second(&mut self) {
        if self.halted() {
            return;
        }

        self.sec += 1;
        if self.sec < 60 {
            return;
        }
        self.sec = 0;

        self.min += 1;
        if self.min < 60 {
            return;
        }
        self.min = 0;

        self.hour += 1;
        if self.hour < 24 {
            return;
        }
        self.hour = 0;

        let mut day = self.day_counter() + 1;
        if day > 0x01FF {
            day = 0;
            self.day_high |= 0x80;
        }
        self.set_day_counter(day);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Mbc3 {
    ram_enabled: bool,
    rom_bank: u8,
    ram_rtc_select: u8,
    latch_last_write: u8,
    rtc_live: Rtc,
    rtc_latched: Option<Rtc>,
    rtc_cycle_accum: u32,
}

impl Mbc3 {
    pub fn new() -> Self {
        Mbc3 {
            ram_enabled: false,
            rom_bank: 1,
            ram_rtc_select: 0,
            latch_last_write: 0xFF,
            rtc_live: Rtc::default(),
            rtc_latched: None,
            rtc_cycle_accum: 0,
        }
    }
}

impl Default for Mbc3 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mbc for Mbc3 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        let bank_size = 0x4000;
        let bank_count = (rom.len() / bank_size).max(1);

        let offset = if addr < 0x4000 {
            // 0x0000..=0x3FFF: fixed bank 0
            addr as usize
        } else {
            // 0x4000..=0x7FFF: switchable bank
            let bank = (self.rom_bank as usize).max(1) % bank_count;
            bank * bank_size + (addr as usize - bank_size)
        };

        rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = val & 0x7F;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
            0x4000..=0x5FFF => {
                self.ram_rtc_select = val & 0x0F;
            }
            0x6000..=0x7FFF => {
                if self.latch_last_write == 0 && val == 1 {
                    self.rtc_latched = Some(self.rtc_live);
                }
                self.latch_last_write = val;
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram: &[u8], addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        // 0x00..=0x03 select RAM bank, 0x08..=0x0C select RTC registers.
        match self.ram_rtc_select {
            0x00..=0x03 => {
                if ram.is_empty() {
                    return 0xFF;
                }

                let bank_size = 0x2000;
                let bank_count = (ram.len() / bank_size).max(1);
                let bank = (self.ram_rtc_select as usize) % bank_count;

                let offset = bank * bank_size + addr.wrapping_sub(0xA000) as usize;
                ram.get(offset).copied().unwrap_or(0xFF)
            }
            0x08..=0x0C => self
                .rtc_latched
                .unwrap_or(self.rtc_live)
                .read_reg(self.ram_rtc_select),
            _ => 0xFF,
        }
    }

    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8) {
        if !self.ram_enabled {
            return;
        }

        match self.ram_rtc_select {
            0x00..=0x03 => {
                if ram.is_empty() {
                    return;
                }

                let bank_size = 0x2000;
                let bank_count = (ram.len() / bank_size).max(1);
                let bank = (self.ram_rtc_select as usize) % bank_count;

                let offset = bank * bank_size + addr.wrapping_sub(0xA000) as usize;
                if let Some(entry) = ram.get_mut(offset) {
                    *entry = val;
                }
            }
            0x08..=0x0C => {
                self.rtc_live.write_reg(self.ram_rtc_select, val);
            }
            _ => {}
        }
    }

    fn tick(&mut self, cycles: u32) {
        if self.rtc_live.halted() {
            return;
        }

        self.rtc_cycle_accum = self.rtc_cycle_accum.saturating_add(cycles);
        while self.rtc_cycle_accum >= CYCLES_PER_SECOND {
            self.rtc_cycle_accum -= CYCLES_PER_SECOND;
            self.rtc_live.increment_second();
        }
    }

    fn save_extra(&self) -> Vec<u8> {
        vec![
            self.rtc_live.sec,
            self.rtc_live.min,
            self.rtc_live.hour,
            self.rtc_live.day_low,
            self.rtc_live.day_high,
            (self.rtc_cycle_accum & 0xFF) as u8,
            ((self.rtc_cycle_accum >> 8) & 0xFF) as u8,
            ((self.rtc_cycle_accum >> 16) & 0xFF) as u8,
            ((self.rtc_cycle_accum >> 24) & 0xFF) as u8,
        ]
    }

    fn load_extra(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            return Ok(());
        }
        if data.len() != 9 {
            return Err("invalid MBC3 RTC payload length");
        }

        self.rtc_live.sec = data[0] % 60;
        self.rtc_live.min = data[1] % 60;
        self.rtc_live.hour = data[2] % 24;
        self.rtc_live.day_low = data[3];
        self.rtc_live.day_high = data[4] & 0xC1;
        self.rtc_cycle_accum = u32::from(data[5])
            | (u32::from(data[6]) << 8)
            | (u32::from(data[7]) << 16)
            | (u32::from(data[8]) << 24);
        self.rtc_cycle_accum %= CYCLES_PER_SECOND;
        self.rtc_latched = None;
        Ok(())
    }
}
