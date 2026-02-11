#[derive(Debug, Default)]
pub struct Serial {
    output: Vec<u8>,
}

impl Serial {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on_transfer(&mut self, byte: u8) {
        self.output.push(byte);
    }

    pub fn drain_output(&mut self) -> std::vec::Drain<'_, u8> {
        self.output.drain(..)
    }

    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }
}
