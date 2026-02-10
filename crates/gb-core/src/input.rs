pub struct Joypad {}

impl Joypad {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}
