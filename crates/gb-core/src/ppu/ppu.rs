use super::{Framebuffer, LCD_HEIGHT, LCD_WIDTH};

pub struct Ppu {
    framebuffer: Framebuffer,
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
            framebuffer: [super::render::DMG_SHADES[0]; LCD_WIDTH * LCD_HEIGHT],
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

    fn clear_framebuffer(&mut self) {
        self.framebuffer.fill(super::render::DMG_SHADES[0]);
    }

    pub fn tick(
        &mut self,
        mut cycles: u32,
        vram: &[u8; 0x2000],
        oam: &[u8; 0xA0],
        io: &mut [u8; 0x80],
        iflag: &mut u8,
    ) {
        let enabled = (io[Self::LCDC] & 0x80) != 0;
        if !enabled {
            if self.lcd_enabled {
                self.clear_framebuffer();
            }
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
            self.dots = 4;
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
                    super::render::render_scanline(&mut self.framebuffer, self.ly, vram, oam, io);
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

    pub fn framebuffer(&self) -> &Framebuffer {
        &self.framebuffer
    }

    pub fn frame_ready(&self) -> bool {
        self.frame_ready
    }

    pub fn clear_frame_ready(&mut self) {
        self.frame_ready = false;
    }

    pub fn current_mode(&self) -> u8 {
        self.mode
    }

    pub fn current_ly(&self) -> u8 {
        self.ly
    }

    pub fn current_dots(&self) -> u32 {
        self.dots
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
        let vram = [0u8; 0x2000];
        let oam = [0u8; 0xA0];
        ppu.tick(456 * 10, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(io[LY], 0);
        assert_eq!(mode(io[STAT]), 0);
        assert_eq!(iflag, 0);
    }

    #[test]
    fn ppu_disabling_lcd_clears_framebuffer() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;
        let mut vram = [0u8; 0x2000];
        let oam = [0u8; 0xA0];

        // Render a non-white pixel first.
        for row in 0..8 {
            vram[16 + row * 2] = 0xFF;
            vram[16 + row * 2 + 1] = 0xFF;
        }
        vram[0x1800] = 1;
        io[0x47] = 0xE4;
        io[LCDC] = 0x91;
        ppu.tick(252, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(ppu.framebuffer()[0], 0xFF000000);

        io[LCDC] = 0x00;
        ppu.tick(4, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(ppu.framebuffer()[0], 0xFFFFFFFF);
    }

    #[test]
    fn ppu_visible_scanline_mode_transitions() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80; // LCD on
        let vram = [0u8; 0x2000];
        let oam = [0u8; 0xA0];
        ppu.tick(0, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(io[LY], 0);
        assert_eq!(mode(io[STAT]), 2);

        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(mode(io[STAT]), 3);

        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(mode(io[STAT]), 0);

        ppu.tick(204, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(io[LY], 1);
        assert_eq!(mode(io[STAT]), 2);
    }

    #[test]
    fn ppu_enters_vblank_and_requests_interrupt() {
        let mut ppu = Ppu::new();
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80;
        let vram = [0u8; 0x2000];
        let oam = [0u8; 0xA0];
        ppu.tick(456 * 144, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(io[LY], 144);
        assert_eq!(mode(io[STAT]), 1);
        assert_ne!(iflag & 0x01, 0); // VBlank interrupt requested
    }

    #[test]
    fn ppu_lyc_coincidence_sets_stat_and_interrupts_on_edge() {
        let mut ppu = Ppu::new();
        let vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        io[LCDC] = 0x80;
        io[LYC] = 1;
        io[STAT] = 0x40; // enable LYC=LY interrupt (bit 6)

        // Advance to LY=1
        let oam = [0u8; 0xA0];
        ppu.tick(456, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(io[LY], 1);
        assert_ne!(io[STAT] & 0x04, 0); // coincidence flag
        assert_ne!(iflag & 0x02, 0); // STAT interrupt requested

        // Still coincident; should not re-trigger every tick.
        iflag = 0;
        ppu.tick(1, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(iflag & 0x02, 0);
    }

    #[test]
    fn ppu_exposes_framebuffer_and_renders_bg() {
        use crate::ppu::{LCD_HEIGHT, LCD_WIDTH};

        let mut ppu = Ppu::new();
        let mut vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        const TILE_1: usize = 16;

        // Tile 1: all pixels color 3.
        for row in 0..8 {
            vram[TILE_1 + row * 2] = 0xFF;
            vram[TILE_1 + row * 2 + 1] = 0xFF;
        }
        // BG map (0x9800): top-left tile is 1.
        vram[0x1800] = 1;

        io[0x47] = 0xE4; // identity palette mapping
        io[0x40] = 0x91; // LCD on, BG on, unsigned tile data, 0x9800 map

        assert_eq!(ppu.framebuffer().len(), LCD_WIDTH * LCD_HEIGHT);

        // Render LY=0 scanline.
        let oam = [0u8; 0xA0];
        ppu.tick(0, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(ppu.framebuffer()[0], 0xFF000000);
        assert_eq!(ppu.framebuffer()[8], 0xFFFFFFFF);
    }

    #[test]
    fn ppu_framebuffer_updates_when_vram_changes() {
        use crate::ppu::LCD_WIDTH;

        let mut ppu = Ppu::new();
        let mut vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        const TILE_1: usize = 16;

        // Tile 1: all pixels color 3.
        for row in 0..8 {
            vram[TILE_1 + row * 2] = 0xFF;
            vram[TILE_1 + row * 2 + 1] = 0xFF;
        }
        vram[0x1800] = 1;

        io[0x47] = 0xE4;
        io[0x40] = 0x91;

        // Render LY=0.
        let oam = [0u8; 0xA0];
        ppu.tick(0, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(ppu.framebuffer()[0], 0xFF000000);

        // Finish line 0 to advance to LY=1.
        ppu.tick(204, &vram, &oam, &mut io, &mut iflag);

        // Change tile 1 to color 0 before rendering next scanline.
        for row in 0..8 {
            vram[TILE_1 + row * 2] = 0x00;
            vram[TILE_1 + row * 2 + 1] = 0x00;
        }

        // Render LY=1.
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(ppu.framebuffer()[LCD_WIDTH], 0xFFFFFFFF);
    }

    #[test]
    fn ppu_scanline_uses_mode3_entry_state() {
        let mut ppu = Ppu::new();
        let mut vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;
        let oam = [0u8; 0xA0];

        // Tile 1 row produces color 1 on first pixel.
        vram[16] = 0x80;
        vram[17] = 0x00;
        vram[0x1800] = 1;

        io[0x40] = 0x91;
        io[0x47] = 0xE4; // identity

        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        // Change palette mid-mode-3; should not retroactively change rendered line.
        io[0x47] = 0x1B;
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(ppu.framebuffer()[0], 0xFFAAAAAA);
    }

    #[test]
    fn ppu_renders_window_over_background() {
        let mut ppu = Ppu::new();
        let mut vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        const TILE_1: usize = 16;

        // Tile 1: all pixels color 3.
        for row in 0..8 {
            vram[TILE_1 + row * 2] = 0xFF;
            vram[TILE_1 + row * 2 + 1] = 0xFF;
        }

        // Window map (0x9C00): top-left tile is 1.
        vram[0x1C00] = 1;

        io[0x47] = 0xE4; // identity palette mapping
        io[0x4A] = 0; // WY
        io[0x4B] = 7; // WX (7 => x=0)
        io[0x40] = 0xF1; // LCD on, BG on, window on, unsigned tile data, window map 0x9C00

        // Render LY=0 scanline.
        let oam = [0u8; 0xA0];
        ppu.tick(0, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);

        // BG defaults to color 0 (white); window should overwrite it to color 3 (black).
        assert_eq!(ppu.framebuffer()[0], 0xFF000000);
    }

    #[test]
    fn ppu_window_respects_wx_wy_and_tilemap_selection() {
        use crate::ppu::LCD_WIDTH;

        let mut ppu = Ppu::new();
        let mut vram = [0u8; 0x2000];
        let mut io = [0u8; 0x80];
        let mut iflag = 0u8;

        const TILE_1: usize = 16;

        // Tile 1: all pixels color 3.
        for row in 0..8 {
            vram[TILE_1 + row * 2] = 0xFF;
            vram[TILE_1 + row * 2 + 1] = 0xFF;
        }

        // Window map (0x9800): top-left tile is 1.
        vram[0x1800] = 1;
        // BG map uses 0x9C00; leave it as tile 0 (white).

        io[0x47] = 0xE4;
        io[0x4A] = 1; // WY
        io[0x4B] = 15; // WX (15 => x=8)
        io[0x40] = 0xB9; // LCD on, BG on, window on, unsigned tile data, BG map 0x9C00, window map 0x9800

        // Render LY=0 (window not active yet).
        let oam = [0u8; 0xA0];
        ppu.tick(0, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);
        assert_eq!(ppu.framebuffer()[8], 0xFFFFFFFF);

        // Advance to LY=1.
        ppu.tick(204, &vram, &oam, &mut io, &mut iflag);

        // Render LY=1 (window active; starts at x=8).
        ppu.tick(80, &vram, &oam, &mut io, &mut iflag);
        ppu.tick(172, &vram, &oam, &mut io, &mut iflag);

        assert_eq!(ppu.framebuffer()[LCD_WIDTH + 7], 0xFFFFFFFF);
        assert_eq!(ppu.framebuffer()[LCD_WIDTH + 8], 0xFF000000);
    }
}
