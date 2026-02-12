use serde::{Deserialize, Serialize};

use super::channels::noise::NoiseChannel;
use super::channels::square::SquareChannel;
use super::channels::wave::WaveChannel;

const CPU_CLOCK_HZ: u64 = 4_194_304;
const FRAME_SEQUENCER_PERIOD_CYCLES: u16 = 8_192;

const NR10: u16 = 0xFF10;
const NR11: u16 = 0xFF11;
const NR12: u16 = 0xFF12;
const NR13: u16 = 0xFF13;
const NR14: u16 = 0xFF14;
const NR21: u16 = 0xFF16;
const NR22: u16 = 0xFF17;
const NR23: u16 = 0xFF18;
const NR24: u16 = 0xFF19;
const NR30: u16 = 0xFF1A;
const NR31: u16 = 0xFF1B;
const NR32: u16 = 0xFF1C;
const NR33: u16 = 0xFF1D;
const NR34: u16 = 0xFF1E;
const NR41: u16 = 0xFF20;
const NR42: u16 = 0xFF21;
const NR43: u16 = 0xFF22;
const NR44: u16 = 0xFF23;
const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;
const WAVE_RAM_START: u16 = 0xFF30;
const WAVE_RAM_END: u16 = 0xFF3F;

#[derive(Serialize, Deserialize)]
pub struct Apu {
    powered: bool,

    cgb_mode: bool,

    ch1: SquareChannel,
    ch2: SquareChannel,
    ch3: WaveChannel,
    ch4: NoiseChannel,

    nr50: u8,
    nr51: u8,

    frame_seq_step: u8,
    frame_seq_counter: u16,

    sample_accum: u64,
    samples: Vec<f32>,
}

impl Apu {
    pub const DEFAULT_SAMPLE_RATE_HZ: u32 = 48_000;
    pub const DEFAULT_CHANNELS: u8 = 2;

    pub fn new() -> Self {
        Self {
            powered: true,
            cgb_mode: false,
            ch1: SquareChannel::new(true),
            ch2: SquareChannel::new(false),
            ch3: WaveChannel::new(),
            ch4: NoiseChannel::new(),
            nr50: 0,
            nr51: 0,
            frame_seq_step: 0,
            frame_seq_counter: 0,
            sample_accum: 0,
            samples: Vec::new(),
        }
    }

    pub fn set_cgb_mode(&mut self, cgb_mode: bool) {
        self.cgb_mode = cgb_mode;
    }

    pub fn tick(&mut self, cycles: u32) {
        for _ in 0..cycles {
            self.tick_cycle();
        }
    }

    fn tick_cycle(&mut self) {
        if self.powered {
            self.ch1.tick_timer();
            self.ch2.tick_timer();
            self.ch3.tick_timer();
            self.ch4.tick_timer();
        }

        // The frame sequencer is only active while the APU is powered on (NR52 bit 7).
        // CGB differs from DMG in how power cycling affects phase, but the sequencer itself
        // is still halted while powered off.
        if self.powered {
            self.frame_seq_counter = self.frame_seq_counter.wrapping_add(1);
            if self.frame_seq_counter >= FRAME_SEQUENCER_PERIOD_CYCLES {
                self.frame_seq_counter = 0;
                self.clock_frame_sequencer();
            }
        }

        self.sample_accum = self
            .sample_accum
            .saturating_add(u64::from(Self::DEFAULT_SAMPLE_RATE_HZ));
        if self.sample_accum >= CPU_CLOCK_HZ {
            self.sample_accum -= CPU_CLOCK_HZ;
            let (left, right) = self.mix_stereo();
            self.samples.push(left);
            self.samples.push(right);
        }
    }

    fn clock_frame_sequencer(&mut self) {
        let step = self.frame_seq_step;
        self.frame_seq_step = (self.frame_seq_step + 1) & 7;

        if self.powered && step.is_multiple_of(2) {
            self.ch1.clock_length_internal(false, self.cgb_mode);
            self.ch2.clock_length_internal(false, self.cgb_mode);
            self.ch3.clock_length_internal(false, self.cgb_mode);
            self.ch4.clock_length_internal(false, self.cgb_mode);
        }

        if !self.powered {
            return;
        }

        if step == 2 || step == 6 {
            self.ch1.clock_sweep();
        }

        if step == 7 {
            self.ch1.clock_envelope();
            self.ch2.clock_envelope();
            self.ch4.clock_envelope();
        }
    }

    fn mix_stereo(&self) -> (f32, f32) {
        if !self.powered {
            return (0.0, 0.0);
        }

        let c1 = self.ch1.output();
        let c2 = self.ch2.output();
        let c3 = self.ch3.output();
        let c4 = self.ch4.output();

        let right_mix = self.route_mix(false, c1, c2, c3, c4);
        let left_mix = self.route_mix(true, c1, c2, c3, c4);

        let left_vol = ((self.nr50 >> 4) & 0x07) as f32;
        let right_vol = (self.nr50 & 0x07) as f32;

        let left = (left_mix / 4.0) * ((left_vol + 1.0) / 8.0);
        let right = (right_mix / 4.0) * ((right_vol + 1.0) / 8.0);

        (left.clamp(-1.0, 1.0), right.clamp(-1.0, 1.0))
    }

    fn route_mix(&self, left: bool, c1: f32, c2: f32, c3: f32, c4: f32) -> f32 {
        let shift = if left { 4 } else { 0 };
        let route = self.nr51 >> shift;

        let mut mix = 0.0;
        if (route & 0x01) != 0 {
            mix += c1;
        }
        if (route & 0x02) != 0 {
            mix += c2;
        }
        if (route & 0x04) != 0 {
            mix += c3;
        }
        if (route & 0x08) != 0 {
            mix += c4;
        }

        mix
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            NR10 => self.ch1.sweep | 0x80,
            NR11 => self.ch1.duty_length | 0x3F,
            NR12 => self.ch1.envelope,
            NR13 => 0xFF,
            NR14 => self.ch1.freq_hi | 0xBF,
            0xFF15 => 0xFF,

            NR21 => self.ch2.duty_length | 0x3F,
            NR22 => self.ch2.envelope,
            NR23 => 0xFF,
            NR24 => self.ch2.freq_hi | 0xBF,
            0xFF1F => 0xFF,

            NR30 => self.ch3.nr30 | 0x7F,
            NR31 => 0xFF,
            NR32 => self.ch3.nr32 | 0x9F,
            NR33 => 0xFF,
            NR34 => self.ch3.nr34 | 0xBF,

            NR41 => 0xFF,
            NR42 => self.ch4.nr42,
            NR43 => self.ch4.nr43,
            NR44 => self.ch4.nr44 | 0xBF,

            NR50 => self.nr50,
            NR51 => self.nr51,
            NR52 => self.nr52_read(),

            WAVE_RAM_START..=WAVE_RAM_END => {
                let index = (addr - WAVE_RAM_START) as usize;
                self.ch3.read_wave_ram(index, self.cgb_mode)
            }

            0xFF27..=0xFF2F => 0xFF,
            _ => 0xFF,
        }
    }

    fn nr52_read(&self) -> u8 {
        let mut status = 0u8;
        if self.powered {
            status |= 0x80;
            if self.ch1.enabled {
                status |= 0x01;
            }
            if self.ch2.enabled {
                status |= 0x02;
            }
            if self.ch3.enabled {
                status |= 0x04;
            }
            if self.ch4.enabled {
                status |= 0x08;
            }
        }

        status | 0x70
    }

    pub fn write_register(&mut self, addr: u16, value: u8, div_counter: u16) {
        if (WAVE_RAM_START..=WAVE_RAM_END).contains(&addr) {
            let index = (addr - WAVE_RAM_START) as usize;
            self.ch3.write_wave_ram(index, value, self.cgb_mode);
            return;
        }

        if addr == NR52 {
            self.write_nr52(value, div_counter);
            return;
        }

        if !self.powered {
            if !self.cgb_mode {
                match addr {
                    NR11 => self.ch1.write_duty_length(value & 0x3F),
                    NR21 => self.ch2.write_duty_length(value & 0x3F),
                    NR31 => self.ch3.write_nr31(value),
                    NR41 => self.ch4.write_nr41(value & 0x3F),
                    _ => {}
                }
            }
            return;
        }

        match addr {
            NR10 => self.ch1.write_sweep(value & 0x7F),
            NR11 => self.ch1.write_duty_length(value),
            NR12 => self.ch1.write_envelope(value),
            NR13 => self.ch1.write_freq_lo(value),
            NR14 => self
                .ch1
                .write_nr14(value, self.frame_seq_step, self.cgb_mode),

            NR21 => self.ch2.write_duty_length(value),
            NR22 => self.ch2.write_envelope(value),
            NR23 => self.ch2.write_freq_lo(value),
            NR24 => self
                .ch2
                .write_nr14(value, self.frame_seq_step, self.cgb_mode),

            NR30 => self.ch3.write_nr30(value),
            NR31 => self.ch3.write_nr31(value),
            NR32 => self.ch3.write_nr32(value),
            NR33 => self.ch3.write_nr33(value, self.cgb_mode),
            NR34 => self
                .ch3
                .write_nr34(value, self.frame_seq_step, self.cgb_mode),

            NR41 => self.ch4.write_nr41(value),
            NR42 => self.ch4.write_nr42(value),
            NR43 => self.ch4.write_nr43(value),
            NR44 => self
                .ch4
                .write_nr44(value, self.frame_seq_step, self.cgb_mode),

            NR50 => self.nr50 = value,
            NR51 => self.nr51 = value,

            _ => {}
        }
    }

    fn write_nr52(&mut self, value: u8, div_counter: u16) {
        let next_power = (value & 0x80) != 0;

        if self.powered && !next_power {
            self.powered = false;
            self.nr50 = 0;
            self.nr51 = 0;

            // On DMG, the frame sequencer is reset when powered off.
            // On CGB, it keeps running.
            if !self.cgb_mode {
                self.frame_seq_step = 0;
                self.frame_seq_counter = 0;
            }

            self.ch1.powered_register_clear(self.cgb_mode);
            self.ch2.powered_register_clear(self.cgb_mode);
            self.ch3.powered_register_clear(self.cgb_mode);
            self.ch4.powered_register_clear(self.cgb_mode);
        } else if !self.powered && next_power {
            self.powered = true;

            // Hardware behavior (cgb_sound test #5): the APU frame sequencer is derived from the
            // global divider, so powering up re-phases the "time to next frame tick".
            // Model this by syncing our sub-counter to DIV's lower 13 bits (mod 8192) while
            // resetting the step to the power-on state.
            if self.cgb_mode {
                // Powering up resets the frame sequencer step to 0.
                self.frame_seq_counter = div_counter & 0x1FFF;
                self.frame_seq_step = 0;
            }

            // On DMG, the frame sequencer is reset when powered on.
            // On CGB, the sequencer step resets to its power-on state, but its sub-cycle phase
            // is effectively aligned to DIV (handled above).
            if !self.cgb_mode {
                self.frame_seq_step = 0;
                self.frame_seq_counter = 0;
            }
        }
    }

    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.samples)
    }

    #[cfg(test)]
    pub fn channel_lengths(&self) -> (u16, u16, u16, u16) {
        (
            self.ch1.length_counter(),
            self.ch2.length_counter(),
            self.ch3.length_counter(),
            self.ch4.length_counter(),
        )
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}
