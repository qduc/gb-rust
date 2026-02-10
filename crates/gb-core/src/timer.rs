pub struct Timer {}

impl Timer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, _cycles: u32, _iflag: &mut u8) {
    }
}
