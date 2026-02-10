use crate::cartridge::Cartridge;
use crate::ppu::Ppu;
use crate::apu::Apu;
use crate::timer::Timer;
use crate::input::Joypad;

pub struct Bus {
    pub cart: Cartridge,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub input: Joypad,
    pub wram: [u8; 0x2000],
    pub hram: [u8; 0x7F],
    pub ie: u8,
    pub iflag: u8,
}

impl Bus {
    pub fn read8(&mut self, _addr: u16) -> u8 {
        0
    }

    pub fn write8(&mut self, _addr: u16, _val: u8) {
    }

    pub fn tick(&mut self, cycles: u32) {
        self.timer.tick(cycles, &mut self.iflag);
        self.ppu.tick(cycles, &mut self.iflag);
        self.apu.tick(cycles);
    }
}
