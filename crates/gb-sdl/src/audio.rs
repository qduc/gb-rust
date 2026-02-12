use sdl2::audio::{AudioQueue, AudioSpecDesired};

pub struct SdlAudio {
    queue: AudioQueue<f32>,
    sample_rate_hz: u32,
    channels: u8,
}

impl SdlAudio {
    pub fn new(
        audio: &sdl2::AudioSubsystem,
        sample_rate_hz: i32,
        channels: u8,
    ) -> Result<Self, String> {
        let sample_rate_hz = u32::try_from(sample_rate_hz)
            .map_err(|_| format!("invalid sample rate: {sample_rate_hz}"))?;
        let desired = AudioSpecDesired {
            freq: Some(sample_rate_hz as i32),
            channels: Some(channels),
            samples: None,
        };

        let queue = audio.open_queue::<f32, _>(None, &desired)?;
        queue.resume();

        Ok(Self {
            queue,
            sample_rate_hz,
            channels,
        })
    }

    pub fn enqueue(&self, samples: &[f32]) -> Result<(), String> {
        self.queue.queue_audio(samples)
    }

    pub fn queued_bytes(&self) -> u32 {
        self.queue.size()
    }

    pub fn clear(&self) {
        self.queue.clear();
    }

    pub fn max_queue_bytes(&self, max_queue_ms: u32) -> u32 {
        let bytes_per_sample = std::mem::size_of::<f32>() as u32;
        self.sample_rate_hz
            .saturating_mul(self.channels as u32)
            .saturating_mul(bytes_per_sample)
            .saturating_mul(max_queue_ms)
            / 1000
    }
}

pub fn pump_apu_to_sdl(
    apu: &mut gb_core::apu::Apu,
    audio: &SdlAudio,
    volume: f32,
) -> Result<(), String> {
    let mut samples = apu.take_samples();
    if samples.is_empty() {
        return Ok(());
    }

    let volume = volume.clamp(0.0, 2.0);
    if (volume - 1.0).abs() > f32::EPSILON {
        for sample in &mut samples {
            *sample = (*sample * volume).clamp(-1.0, 1.0);
        }
    }

    const MAX_QUEUE_MS: u32 = 120;
    let max_queue_bytes = audio.max_queue_bytes(MAX_QUEUE_MS);

    if audio.queued_bytes() > max_queue_bytes {
        audio.clear();
    }

    let queued_bytes = audio.queued_bytes();
    if queued_bytes >= max_queue_bytes {
        return Ok(());
    }

    let bytes_per_sample = std::mem::size_of::<f32>();
    let remaining_samples = ((max_queue_bytes - queued_bytes) as usize) / bytes_per_sample;
    if remaining_samples == 0 {
        return Ok(());
    }

    if samples.len() > remaining_samples {
        samples = samples[samples.len() - remaining_samples..].to_vec();
    }

    audio.enqueue(&samples)
}
