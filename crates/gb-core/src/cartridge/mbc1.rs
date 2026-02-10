use crate::cartridge::mbc::Mbc;

pub struct Mbc1;

impl Mbc for Mbc1 {
    fn read_rom(&self, _rom: &[u8], _addr: u16) -> u8 {
        0
    }
    fn write_rom(&mut self, _addr: u16, _val: u8) {}
    fn read_ram(&self, _ram: &[u8], _addr: u16) -> u8 {
        0
    }
    fn write_ram(&mut self, _ram: &mut [u8], _addr: u16, _val: u8) {}
}
