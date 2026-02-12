use serde::{Deserialize, Serialize};

const SERIAL_INTERNAL_TRANSFER_CYCLES: u32 = 4096;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Serial {
    output: Vec<u8>,
    in_progress: bool,
    cycles_remaining: u32,
    pending_byte: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_transfer(&mut self, byte: u8) {
        self.output.push(byte);
    }

    pub fn start_transfer(&mut self, byte: u8, sc: &mut u8) {
        self.pending_byte = byte;
        self.in_progress = (*sc & 0x80) != 0;
        let internal_clock = (*sc & 0x01) != 0;
        self.cycles_remaining = if self.in_progress && internal_clock {
            SERIAL_INTERNAL_TRANSFER_CYCLES
        } else {
            0
        };
        *sc |= 0x80;
    }

    pub fn stop_transfer(&mut self, sc: &mut u8) {
        self.in_progress = false;
        self.cycles_remaining = 0;
        *sc &= 0x7F;
    }

    pub fn tick(&mut self, cycles: u32, iflag: &mut u8, sc: &mut u8) {
        if !self.in_progress {
            return;
        }

        if self.cycles_remaining == 0 || cycles >= self.cycles_remaining {
            self.in_progress = false;
            self.cycles_remaining = 0;
            *sc &= 0x7F;
            self.on_transfer(self.pending_byte);
            *iflag |= crate::interrupt::Interrupt::Serial.bit();
        } else {
            self.cycles_remaining -= cycles;
        }
    }

    pub fn drain_output(&mut self) -> std::vec::Drain<'_, u8> {
        self.output.drain(..)
    }

    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }
}
