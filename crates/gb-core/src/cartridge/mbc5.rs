use crate::cartridge::mbc::Mbc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mbc5 {
    ram_enabled: bool,
    rom_bank: u16,
    ram_bank: u8,
}

impl Mbc5 {
    pub fn new() -> Self {
        Self {
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
        }
    }
}

impl Default for Mbc5 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mbc for Mbc5 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        let bank_size = 0x4000;
        let bank_count = (rom.len() / bank_size).max(1);

        let offset = if addr < 0x4000 {
            addr as usize
        } else {
            let bank = (self.rom_bank as usize) % bank_count;
            bank * bank_size + (addr as usize - bank_size)
        };

        rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x0100) | val as u16;
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x00FF) | (((val & 0x01) as u16) << 8);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = val & 0x0F;
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
        let bank = (self.ram_bank as usize) % bank_count;
        let offset = bank * bank_size + addr.wrapping_sub(0xA000) as usize;
        ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8) {
        if !self.ram_enabled || ram.is_empty() {
            return;
        }

        let bank_size = 0x2000;
        let bank_count = (ram.len() / bank_size).max(1);
        let bank = (self.ram_bank as usize) % bank_count;
        let offset = bank * bank_size + addr.wrapping_sub(0xA000) as usize;

        if let Some(entry) = ram.get_mut(offset) {
            *entry = val;
        }
    }
}
