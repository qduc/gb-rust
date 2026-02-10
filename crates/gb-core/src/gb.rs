use crate::cpu::Cpu;
use crate::bus::Bus;

pub struct GameBoy {
    pub cpu: Cpu,
    pub bus: Bus,
}

impl GameBoy {
    pub fn step(&mut self) -> u32 {
        let cycles = self.cpu.step(&mut self.bus);
        self.bus.tick(cycles);
        cycles
    }

    pub fn run_frame(&mut self) {
        while !self.bus.ppu.frame_ready() {
            self.step();
        }
        self.bus.ppu.clear_frame_ready();
    }
}
