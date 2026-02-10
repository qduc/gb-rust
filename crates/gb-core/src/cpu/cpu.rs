use crate::bus::Bus;

pub struct Cpu {
    // registers, flags, etc.
}

impl Cpu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&mut self, _bus: &mut Bus) -> u32 {
        // execute one instruction
        4
    }
}
