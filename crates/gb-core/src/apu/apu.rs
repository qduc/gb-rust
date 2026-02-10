pub struct Apu {}

impl Apu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, _cycles: u32) {}
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}
