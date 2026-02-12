use crate::bus::Bus;
use crate::cpu::Cpu;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GameBoy {
    pub cpu: Cpu,
    pub bus: Bus,
}

impl GameBoy {
    pub fn step(&mut self) -> u32 {
        self.cpu.step(&mut self.bus)
    }

    pub fn run_frame(&mut self) {
        while !self.bus.ppu.frame_ready() {
            self.step();
        }
        self.bus.ppu.clear_frame_ready();
    }
}
