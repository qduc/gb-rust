use crate::cartridge::mbc::Mbc;
use serde::{Deserialize, Serialize};

const RAM_SIZE: usize = 0x200;

#[derive(Serialize, Deserialize)]
pub struct Mbc2 {
    ram_enabled: bool,
    rom_bank: u8,
    #[serde(with = "serde_bytes")]
    ram: Vec<u8>,
}

impl Mbc2 {
    pub fn new() -> Self {
        Self {
            ram_enabled: false,
            rom_bank: 1,
            ram: vec![0; RAM_SIZE],
        }
    }

    fn ram_index(addr: u16) -> usize {
        addr.wrapping_sub(0xA000) as usize & 0x01FF
    }
}

impl Default for Mbc2 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mbc for Mbc2 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        let bank_size = 0x4000;
        let bank_count = (rom.len() / bank_size).max(1);

        let offset = if addr < 0x4000 {
            addr as usize
        } else {
            let bank = (self.rom_bank as usize).max(1) % bank_count;
            bank * bank_size + (addr as usize - bank_size)
        };

        rom.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF if (addr & 0x0100) == 0 => {
                self.ram_enabled = (val & 0x0F) == 0x0A;
            }
            0x0000..=0x3FFF => {
                self.rom_bank = val & 0x0F;
                if self.rom_bank == 0 {
                    self.rom_bank = 1;
                }
            }
            _ => {}
        }
    }

    fn read_ram(&self, _ram: &[u8], addr: u16) -> u8 {
        if !self.ram_enabled || !(0xA000..=0xBFFF).contains(&addr) {
            return 0xFF;
        }
        0xF0 | (self.ram[Self::ram_index(addr)] & 0x0F)
    }

    fn write_ram(&mut self, _ram: &mut [u8], addr: u16, val: u8) {
        if !self.ram_enabled || !(0xA000..=0xBFFF).contains(&addr) {
            return;
        }
        self.ram[Self::ram_index(addr)] = val & 0x0F;
    }

    fn save_extra(&self) -> Vec<u8> {
        self.ram.to_vec()
    }

    fn load_extra(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            return Ok(());
        }
        if data.len() != RAM_SIZE {
            return Err("invalid MBC2 save payload length");
        }
        self.ram.copy_from_slice(data);
        Ok(())
    }
}
