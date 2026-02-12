use crate::cartridge::mbc::Mbc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Mbc0;

impl Mbc0 {
    pub fn new() -> Self {
        Mbc0
    }
}

impl Default for Mbc0 {
    fn default() -> Self {
        Self::new()
    }
}

impl Mbc for Mbc0 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        let idx = addr as usize;
        if idx < rom.len() {
            rom[idx]
        } else {
            0xFF
        }
    }

    fn write_rom(&mut self, _addr: u16, _val: u8) {}

    fn read_ram(&self, ram: &[u8], addr: u16) -> u8 {
        if ram.is_empty() {
            return 0xFF;
        }
        let offset = addr.wrapping_sub(0xA000) as usize;
        ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8) {
        if ram.is_empty() {
            return;
        }
        let offset = addr.wrapping_sub(0xA000) as usize;
        if let Some(entry) = ram.get_mut(offset) {
            *entry = val;
        }
    }
}
