use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MbcEnum {
    Mbc0(super::mbc0::Mbc0),
    Mbc1(super::mbc1::Mbc1),
    Mbc2(super::mbc2::Mbc2),
    Mbc3(super::mbc3::Mbc3),
    Mbc5(super::mbc5::Mbc5),
}

impl Mbc for MbcEnum {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8 {
        match self {
            Self::Mbc0(m) => m.read_rom(rom, addr),
            Self::Mbc1(m) => m.read_rom(rom, addr),
            Self::Mbc2(m) => m.read_rom(rom, addr),
            Self::Mbc3(m) => m.read_rom(rom, addr),
            Self::Mbc5(m) => m.read_rom(rom, addr),
        }
    }

    fn write_rom(&mut self, addr: u16, val: u8) {
        match self {
            Self::Mbc0(m) => m.write_rom(addr, val),
            Self::Mbc1(m) => m.write_rom(addr, val),
            Self::Mbc2(m) => m.write_rom(addr, val),
            Self::Mbc3(m) => m.write_rom(addr, val),
            Self::Mbc5(m) => m.write_rom(addr, val),
        }
    }

    fn read_ram(&self, ram: &[u8], addr: u16) -> u8 {
        match self {
            Self::Mbc0(m) => m.read_ram(ram, addr),
            Self::Mbc1(m) => m.read_ram(ram, addr),
            Self::Mbc2(m) => m.read_ram(ram, addr),
            Self::Mbc3(m) => m.read_ram(ram, addr),
            Self::Mbc5(m) => m.read_ram(ram, addr),
        }
    }

    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8) {
        match self {
            Self::Mbc0(m) => m.write_ram(ram, addr, val),
            Self::Mbc1(m) => m.write_ram(ram, addr, val),
            Self::Mbc2(m) => m.write_ram(ram, addr, val),
            Self::Mbc3(m) => m.write_ram(ram, addr, val),
            Self::Mbc5(m) => m.write_ram(ram, addr, val),
        }
    }

    fn tick(&mut self, cycles: u32) {
        match self {
            Self::Mbc0(m) => m.tick(cycles),
            Self::Mbc1(m) => m.tick(cycles),
            Self::Mbc2(m) => m.tick(cycles),
            Self::Mbc3(m) => m.tick(cycles),
            Self::Mbc5(m) => m.tick(cycles),
        }
    }

    fn save_extra(&self) -> Vec<u8> {
        match self {
            Self::Mbc0(m) => m.save_extra(),
            Self::Mbc1(m) => m.save_extra(),
            Self::Mbc2(m) => m.save_extra(),
            Self::Mbc3(m) => m.save_extra(),
            Self::Mbc5(m) => m.save_extra(),
        }
    }

    fn load_extra(&mut self, data: &[u8]) -> Result<(), &'static str> {
        match self {
            Self::Mbc0(m) => m.load_extra(data),
            Self::Mbc1(m) => m.load_extra(data),
            Self::Mbc2(m) => m.load_extra(data),
            Self::Mbc3(m) => m.load_extra(data),
            Self::Mbc5(m) => m.load_extra(data),
        }
    }
}

pub trait Mbc {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);
    fn read_ram(&self, ram: &[u8], addr: u16) -> u8;
    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8);

    fn tick(&mut self, _cycles: u32) {}

    fn save_extra(&self) -> Vec<u8> {
        Vec::new()
    }

    fn load_extra(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.is_empty() {
            Ok(())
        } else {
            Err("unexpected mapper save data")
        }
    }
}
