pub struct Apu {
    samples: Vec<f32>,
}

impl Apu {
    pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;
    pub const DEFAULT_CHANNELS: u8 = 2;

    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn tick(&mut self, _cycles: u32) {
        // TODO: APU not implemented yet.
    }

    /// Drain all currently-produced interleaved stereo samples.
    ///
    /// When the APU is not implemented this returns an empty buffer.
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    /// Push one interleaved stereo sample pair (L, R).
    ///
    /// This is scaffolding for a future APU implementation.
    pub fn push_sample(&mut self, left: f32, right: f32) {
        self.samples.push(left);
        self.samples.push(right);
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}
