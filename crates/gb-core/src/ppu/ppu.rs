pub struct Ppu {
    pub framebuffer: [u32; 160 * 144],
    frame_ready: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            framebuffer: [0; 160 * 144],
            frame_ready: false,
        }
    }

    pub fn tick(&mut self, _cycles: u32, _iflag: &mut u8) {
    }

    pub fn frame_ready(&self) -> bool {
        self.frame_ready
    }

    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }
}
