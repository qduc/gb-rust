pub struct Ppu {
    pub framebuffer: [u32; 160 * 144],
    frame_ready: bool,

    // Phase 6: timing state (rendering comes later)
    dots: u32,
    ly: u8,
    mode: u8,
    lcd_enabled: bool,
    prev_coincidence: bool,
}

impl Ppu {
    const LCDC: usize = 0x40;
    const STAT: usize = 0x41;
    const LY: usize = 0x44;
    const LYC: usize = 0x45;

    const IF_VBLANK: u8 = 0x01;
    const IF_STAT: u8 = 0x02;

    pub fn new() -> Self {
        Self {
            framebuffer: [0; 160 * 144],
            frame_ready: false,
            dots: 0,
            ly: 0,
            mode: 0,
            lcd_enabled: false,
            prev_coincidence: false,
        }
    }

    pub fn reset_ly(&mut self) {
        self.dots = 0;
        self.ly = 0;
        self.mode = if self.lcd_enabled { 2 } else { 0 };
        self.prev_coincidence = false;
        self.frame_ready = false;
    }

    pub fn tick(&mut self, mut cycles: u32, io: &mut [u8; 0x80], iflag: &mut u8) {
        let enabled = (io[Self::LCDC] & 0x80) != 0;
        if !enabled {
            self.lcd_enabled = false;
            self.dots = 0;
            self.ly = 0;
            self.mode = 0;
            self.prev_coincidence = false;
            self.frame_ready = false;
            self.sync_registers(io, iflag);
            return;
        }

        if !self.lcd_enabled {
            self.lcd_enabled = true;
            self.dots = 0;
            self.ly = 0;
            self.mode = 2;
            self.prev_coincidence = false;
        }

        while cycles > 0 {
            let next = self.cycles_to_next_event();
            let step = next.min(cycles);
            self.dots += step;
            cycles -= step;

            // Mode transitions during visible lines.
            if self.ly < 144 {
                if self.mode == 2 && self.dots == 80 {
                    self.set_mode(3, io, iflag);
                } else if self.mode == 3 && self.dots == 252 {
                    self.set_mode(0, io, iflag);
                }
            }

            // End-of-line.
            if self.dots == 456 {
                self.dots = 0;
                self.ly = self.ly.wrapping_add(1);

                if self.ly == 144 {
                    self.frame_ready = true;
                    *iflag |= Self::IF_VBLANK;
                    self.set_mode(1, io, iflag);
                } else if self.ly > 153 {
                    self.ly = 0;
                    self.set_mode(2, io, iflag);
                } else if self.ly >= 144 {
                    self.set_mode(1, io, iflag);
                } else {
                    self.set_mode(2, io, iflag);
                }

                self.sync_registers(io, iflag);
            }
        }

        self.sync_registers(io, iflag);
    }

    fn cycles_to_next_event(&self) -> u32 {
        if self.ly >= 144 {
            456 - self.dots
        } else {
            match self.mode {
                2 => 80 - self.dots,
                3 => 252 - self.dots,
                0 => 456 - self.dots,
                _ => 456 - self.dots,
            }
        }
    }

    fn set_mode(&mut self, mode: u8, io: &mut [u8; 0x80], iflag: &mut u8) {
        if mode == self.mode {
            return;
        }
        self.mode = mode;

        match self.mode {
            0 if (io[Self::STAT] & 0x08) != 0 => *iflag |= Self::IF_STAT,
            1 if (io[Self::STAT] & 0x10) != 0 => *iflag |= Self::IF_STAT,
            2 if (io[Self::STAT] & 0x20) != 0 => *iflag |= Self::IF_STAT,
            _ => {}
        }
    }

    fn sync_registers(&mut self, io: &mut [u8; 0x80], iflag: &mut u8) {
        io[Self::LY] = self.ly;

        let coincidence = self.ly == io[Self::LYC];
        if coincidence && !self.prev_coincidence && (io[Self::STAT] & 0x40) != 0 {
            *iflag |= Self::IF_STAT;
        }
        self.prev_coincidence = coincidence;

        let mut stat = io[Self::STAT] & 0x78; // keep interrupt enables
        stat |= self.mode & 0x03;
        if coincidence {
            stat |= 0x04;
        }
        io[Self::STAT] = stat;
    }

    pub fn frame_ready(&self) -> bool {
        self.frame_ready
    }

    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Ppu;

    const LCDC: usize = 0x40;
    const STAT: usize = 0x41;
    const LY: usize = 0x44;
    const LYC: usize = 0x45;

    fn mode(stat: u8) -> u8 {
        stat & 0x03
    }

    #[test]
    fn ppu_disabled_forces_ly0_mode0() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x00; // LCD off
        ppu.tick(456 * 10, &mut io, &mut iflag);

        assert_eq!(io[LY], 0);
        assert_eq!(mode(io[STAT]), 0);
        assert_eq!(iflag, 0);
    }

    #[test]
    fn ppu_visible_scanline_mode_transitions() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80; // LCD on
        ppu.tick(0, &mut io, &mut iflag);
        assert_eq!(io[LY], 0);
        assert_eq!(mode(io[STAT]), 2);

        ppu.tick(80, &mut io, &mut iflag);
        assert_eq!(mode(io[STAT]), 3);

        ppu.tick(172, &mut io, &mut iflag);
        assert_eq!(mode(io[STAT]), 0);

        ppu.tick(204, &mut io, &mut iflag);
        assert_eq!(io[LY], 1);
        assert_eq!(mode(io[STAT]), 2);
    }

    #[test]
    fn ppu_enters_vblank_and_requests_interrupt() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80;
        ppu.tick(456 * 144, &mut io, &mut iflag);

        assert_eq!(io[LY], 144);
        assert_eq!(mode(io[STAT]), 1);
        assert_ne!(iflag & 0x01, 0); // VBlank interrupt requested
    }

    #[test]
    fn ppu_lyc_coincidence_sets_stat_and_interrupts_on_edge() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80;
        io[LYC] = 1;
        io[STAT] = 0x40; // enable LYC=LY interrupt (bit 6)

        // Advance to LY=1
        ppu.tick(456, &mut io, &mut iflag);

        assert_eq!(io[LY], 1);
        assert_ne!(io[STAT] & 0x04, 0); // coincidence flag
        assert_ne!(iflag & 0x02, 0); // STAT interrupt requested

        // Still coincident; should not re-trigger every tick.
        iflag = 0;
        ppu.tick(1, &mut io, &mut iflag);
        assert_eq!(iflag & 0x02, 0);
    }
}
