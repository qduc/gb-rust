use crate::cartridge::mbc::Mbc;

pub struct Mbc3 {
    ram_enabled: bool,
    rom_bank: u8,
    ram_rtc_select: u8,
}

impl Mbc3 {
    pub fn new() -> Self {
        Mbc3 {
            ram_enabled: false,
            rom_bank: 1,
            ram_rtc_select: 0,
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
            0x08..=0x0C => 0x00, // RTC stub
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
                // RTC stub
            }
            _ => {}
        }
    }
}
