use crate::cartridge::mbc::Mbc;

pub struct Mbc0;

impl Mbc for Mbc0 {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        rom[addr as usize]
    }
    fn write_rom(&mut self, _addr: u16, _val: u8) {}
    fn read_ram(&self, _ram: &[u8], _addr: u16) -> u8 { 0xFF }
    fn write_ram(&mut self, _ram: &mut [u8], _addr: u16, _val: u8) {}
}
