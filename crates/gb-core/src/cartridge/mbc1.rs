use crate::cartridge::mbc::Mbc;

pub struct Mbc1 {
    ram_enabled: bool,
    rom_bank_low5: u8,
    bank_high2: u8,
    banking_mode: u8,
}

impl Mbc1 {
    pub fn new() -> Self {
        Mbc1 {
            ram_enabled: false,
            rom_bank_low5: 1,
            bank_high2: 0,
            banking_mode: 0,
        }
    }
}

impl Default for Mbc1 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mbc for Mbc1 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        let bank_size = 0x4000;
        let bank_count = (rom.len() / bank_size).max(1);

        let offset = if addr < 0x4000 {
            // 0x0000..=0x3FFF: bank 0 in mode 0, or high-bits only in mode 1
            if self.banking_mode == 0 {
                addr as usize
            } else {
                let bank = (self.bank_high2 as usize) << 5;
                (bank * bank_size + addr as usize) % rom.len()
            }
        } else {
            // 0x4000..=0x7FFF: lower bits always from rom_bank_low5, upper from bank_high2
            let bank = ((self.bank_high2 as usize) << 5) | (self.rom_bank_low5 as usize);
            let bank = bank % bank_count;
            bank * bank_size + addr.wrapping_sub(0x4000) as usize
        };

        rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank_low5 = val & 0x1F;
                if self.rom_bank_low5 == 0 {
                    self.rom_bank_low5 = 1;
                }
            }
            0x4000..=0x5FFF => {
                self.bank_high2 = val & 0x03;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = val & 0x01;
            }
            _ => {}
        }
    }

    fn read_ram(&self, ram: &[u8], addr: u16) -> u8 {
        if !self.ram_enabled || ram.is_empty() {
            return 0xFF;
        }

        let bank_size = 0x2000;
        let bank_count = (ram.len() / bank_size).max(1);

        let bank = if self.banking_mode == 0 {
            0
        } else {
            (self.bank_high2 as usize) % bank_count
        };

        let offset = bank * bank_size + (addr.wrapping_sub(0xA000) as usize);
        ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8) {
        if !self.ram_enabled || ram.is_empty() {
            return;
        }

        let bank_size = 0x2000;
        let bank_count = (ram.len() / bank_size).max(1);

        let bank = if self.banking_mode == 0 {
            0
        } else {
            (self.bank_high2 as usize) % bank_count
        };

        let offset = bank * bank_size + (addr.wrapping_sub(0xA000) as usize);
        if let Some(entry) = ram.get_mut(offset) {
            *entry = val;
        }
    }
}
