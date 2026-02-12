use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::dma;
use crate::input::Joypad;
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmulationMode {
    Dmg,
    Cgb,
}

pub struct Bus {
    pub cart: Cartridge,
    pub mode: EmulationMode,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub input: Joypad,
    pub serial: Serial,
    pub wram: [u8; 0x8000],
    pub vram: [u8; 0x4000],
    pub oam: [u8; 0xA0],
    pub io: [u8; 0x80],
    pub hram: [u8; 0x7F],
    pub ie: u8,
    pub iflag: u8,
    pub oam_dma: dma::OamDma,

    // CGB speed switch state (KEY1 / STOP handshake).
    cgb_double_speed: bool,
    cgb_speed_switch_prepare: bool,
    cgb_vram_bank: u8,
    cgb_wram_bank: u8,
    cgb_hdma_src: u16,
    cgb_hdma_dst: u16,
    cgb_hdma_blocks_remaining: u8,
    cgb_hdma_active: bool,
    cgb_hdma_last_hblank_ly: Option<u8>,
    oam_bug_read_idu_pending_addr: Option<u16>,
}

impl Bus {
    const OAM_ROW_BYTES: usize = 8;

    pub fn new(cart: Cartridge) -> Self {
        let mode = match cart.header.cgb_support {
            crate::cartridge::header::CgbSupport::DmgOnly => EmulationMode::Dmg,
            crate::cartridge::header::CgbSupport::CgbCompatible
            | crate::cartridge::header::CgbSupport::CgbOnly => EmulationMode::Cgb,
        };

        let mut apu = Apu::new();
        apu.set_cgb_mode(mode == EmulationMode::Cgb);

        Self {
            cart,
            mode,
            ppu: Ppu::new(),
            apu,
            timer: Timer::new(),
            input: Joypad::new(),
            serial: Serial::new(),
            wram: [0; 0x8000],
            vram: [0; 0x4000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
            iflag: 0,
            oam_dma: dma::OamDma::default(),
            cgb_double_speed: false,
            cgb_speed_switch_prepare: false,
            cgb_vram_bank: 0,
            cgb_wram_bank: 1,
            cgb_hdma_src: 0,
            cgb_hdma_dst: 0x8000,
            cgb_hdma_blocks_remaining: 0,
            cgb_hdma_active: false,
            cgb_hdma_last_hblank_ly: None,
            oam_bug_read_idu_pending_addr: None,
        }
    }

    #[inline]
    fn is_cgb(&self) -> bool {
        self.mode == EmulationMode::Cgb
    }

    fn read_key1(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        let speed = if self.cgb_double_speed { 0x80 } else { 0x00 };
        let prepare = if self.cgb_speed_switch_prepare {
            0x01
        } else {
            0x00
        };
        speed | 0x7E | prepare
    }

    fn read_vbk(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        0xFE | (self.cgb_vram_bank & 0x01)
    }

    fn write_vbk(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        self.cgb_vram_bank = val & 0x01;
    }

    fn read_svbk(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        0xF8 | self.cgb_wram_bank
    }

    fn write_svbk(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        let mut bank = val & 0x07;
        if bank == 0 {
            bank = 1;
        }
        self.cgb_wram_bank = bank;
    }

    fn selected_wram_bank(&self) -> usize {
        if self.is_cgb() {
            self.cgb_wram_bank as usize
        } else {
            1
        }
    }

    fn read_wram(&self, addr: u16) -> u8 {
        match addr {
            0xC000..=0xCFFF => self.wram[(addr - 0xC000) as usize],
            0xD000..=0xDFFF => {
                let bank = self.selected_wram_bank();
                let offset = (addr - 0xD000) as usize;
                self.wram[bank * 0x1000 + offset]
            }
            0xE000..=0xEFFF => self.wram[(addr - 0xE000) as usize],
            0xF000..=0xFDFF => {
                let bank = self.selected_wram_bank();
                let offset = (addr - 0xF000) as usize;
                self.wram[bank * 0x1000 + offset]
            }
            _ => 0xFF,
        }
    }

    fn write_wram(&mut self, addr: u16, val: u8) {
        match addr {
            0xC000..=0xCFFF => self.wram[(addr - 0xC000) as usize] = val,
            0xD000..=0xDFFF => {
                let bank = self.selected_wram_bank();
                let offset = (addr - 0xD000) as usize;
                self.wram[bank * 0x1000 + offset] = val;
            }
            0xE000..=0xEFFF => self.wram[(addr - 0xE000) as usize] = val,
            0xF000..=0xFDFF => {
                let bank = self.selected_wram_bank();
                let offset = (addr - 0xF000) as usize;
                self.wram[bank * 0x1000 + offset] = val;
            }
            _ => {}
        }
    }

    fn write_key1(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        self.cgb_speed_switch_prepare = (val & 0x01) != 0;
    }

    fn read_hdma1(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        (self.cgb_hdma_src >> 8) as u8
    }

    fn read_hdma2(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        (self.cgb_hdma_src as u8) & 0xF0
    }

    fn read_hdma3(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        (((self.cgb_hdma_dst - 0x8000) >> 8) as u8) & 0x1F
    }

    fn read_hdma4(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }
        ((self.cgb_hdma_dst - 0x8000) as u8) & 0xF0
    }

    fn read_hdma5(&self) -> u8 {
        if !self.is_cgb() {
            return 0xFF;
        }

        if !self.cgb_hdma_active && self.cgb_hdma_blocks_remaining == 0 {
            return 0xFF;
        }

        let len = self.cgb_hdma_blocks_remaining.saturating_sub(1) & 0x7F;
        if self.cgb_hdma_active {
            len
        } else {
            0x80 | len
        }
    }

    fn write_hdma1(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        self.cgb_hdma_src = ((val as u16) << 8) | (self.cgb_hdma_src & 0x00F0);
    }

    fn write_hdma2(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        self.cgb_hdma_src = (self.cgb_hdma_src & 0xFF00) | ((val as u16) & 0x00F0);
    }

    fn write_hdma3(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        let upper = ((val & 0x1F) as u16) << 8;
        let lower = (self.cgb_hdma_dst - 0x8000) & 0x00F0;
        self.cgb_hdma_dst = 0x8000 | upper | lower;
    }

    fn write_hdma4(&mut self, val: u8) {
        if !self.is_cgb() {
            return;
        }
        let upper = (self.cgb_hdma_dst - 0x8000) & 0x1F00;
        let lower = (val as u16) & 0x00F0;
        self.cgb_hdma_dst = 0x8000 | upper | lower;
    }

    fn sanitize_hdma_source(addr: u16) -> u16 {
        let masked = addr & 0xFFF0;
        if (0x8000..=0x9FF0).contains(&masked) {
            masked & 0x7FF0
        } else {
            masked
        }
    }

    fn start_hdma_transfer(&mut self, control: u8) {
        if !self.is_cgb() {
            return;
        }

        // Writing bit7=0 while HDMA is active terminates the in-progress HBlank transfer.
        if self.cgb_hdma_active && (control & 0x80) == 0 {
            self.cgb_hdma_active = false;
            self.cgb_hdma_last_hblank_ly = None;
            return;
        }

        self.cgb_hdma_src = Self::sanitize_hdma_source(self.cgb_hdma_src);
        self.cgb_hdma_dst = 0x8000 | ((self.cgb_hdma_dst - 0x8000) & 0x1FF0);
        self.cgb_hdma_blocks_remaining = (control & 0x7F).wrapping_add(1);
        self.cgb_hdma_last_hblank_ly = None;

        if (control & 0x80) == 0 {
            self.cgb_hdma_active = false;
            while self.cgb_hdma_blocks_remaining > 0 {
                self.perform_hdma_block();
            }
        } else {
            self.cgb_hdma_active = true;
        }
    }

    fn perform_hdma_block(&mut self) {
        if self.cgb_hdma_blocks_remaining == 0 {
            self.cgb_hdma_active = false;
            return;
        }

        let src_base = Self::sanitize_hdma_source(self.cgb_hdma_src);
        let dst_base = 0x8000 | ((self.cgb_hdma_dst - 0x8000) & 0x1FF0);

        for i in 0..0x10u16 {
            let v = self.read8_direct(src_base.wrapping_add(i));
            self.write8_direct(dst_base.wrapping_add(i), v);
        }

        self.cgb_hdma_src = Self::sanitize_hdma_source(src_base.wrapping_add(0x10));
        self.cgb_hdma_dst = 0x8000 | (((dst_base - 0x8000).wrapping_add(0x10)) & 0x1FF0);
        self.cgb_hdma_blocks_remaining -= 1;

        if self.cgb_hdma_blocks_remaining == 0 {
            self.cgb_hdma_active = false;
            self.cgb_hdma_last_hblank_ly = None;
        }
    }

    fn tick_hdma(&mut self) {
        if !self.is_cgb() || !self.cgb_hdma_active {
            return;
        }

        // Pragmatic behavior: if LCD is disabled, perform remaining blocks immediately.
        if !self.lcd_enabled() {
            while self.cgb_hdma_blocks_remaining > 0 {
                self.perform_hdma_block();
            }
            return;
        }

        let ly = self.io[0x44];
        let mode = self.ppu_mode();

        if mode == 0 && ly < 144 {
            if self.cgb_hdma_last_hblank_ly != Some(ly) {
                self.perform_hdma_block();
                self.cgb_hdma_last_hblank_ly = Some(ly);
            }
        } else {
            self.cgb_hdma_last_hblank_ly = None;
        }
    }

    fn lcd_enabled(&self) -> bool {
        (self.io[0x40] & 0x80) != 0
    }

    fn ppu_mode(&self) -> u8 {
        self.io[0x41] & 0x03
    }

    fn cpu_access_blocked_by_ppu(&self, addr: u16) -> bool {
        if !self.lcd_enabled() {
            return false;
        }

        let mode = self.ppu_mode();
        match addr {
            // VRAM is inaccessible to CPU during mode 3.
            0x8000..=0x9FFF => mode == 3,
            // OAM is inaccessible to CPU during modes 2 and 3.
            0xFE00..=0xFE9F => mode == 2 || mode == 3,
            _ => false,
        }
    }

    fn oam_bug_active_window(&self) -> bool {
        if !self.lcd_enabled() {
            return false;
        }
        if self.ppu.current_mode() != 2 {
            return false;
        }
        self.ppu.current_ly() < 144
    }

    fn oam_bug_row(&self) -> Option<usize> {
        if !self.oam_bug_active_window() {
            return None;
        }
        let row = (self.ppu.current_dots() / 4) as usize;
        if row < 20 {
            Some(row)
        } else {
            None
        }
    }

    fn oam_word(&self, row: usize, word: usize) -> u16 {
        let base = row * Self::OAM_ROW_BYTES + word * 2;
        u16::from_le_bytes([self.oam[base], self.oam[base + 1]])
    }

    fn set_oam_word(&mut self, row: usize, word: usize, val: u16) {
        let base = row * Self::OAM_ROW_BYTES + word * 2;
        let [lo, hi] = val.to_le_bytes();
        self.oam[base] = lo;
        self.oam[base + 1] = hi;
    }

    fn copy_oam_row_suffix_from_previous(&mut self, row: usize) {
        if row == 0 {
            return;
        }
        for word in 1..4 {
            let v = self.oam_word(row - 1, word);
            self.set_oam_word(row, word, v);
        }
    }

    fn copy_oam_full_row(&mut self, dst_row: usize, src_row: usize) {
        if dst_row >= 20 || src_row >= 20 {
            return;
        }
        let src = src_row * Self::OAM_ROW_BYTES;
        let dst = dst_row * Self::OAM_ROW_BYTES;
        self.oam.copy_within(src..(src + Self::OAM_ROW_BYTES), dst);
    }

    fn apply_oam_bug_write(&mut self, row: usize) {
        if row == 0 || row >= 20 {
            return;
        }
        if row == 1 {
            self.copy_oam_row_suffix_from_previous(row);
            return;
        }
        let a = self.oam_word(row, 0);
        let b = self.oam_word(row - 1, 0);
        let c = self.oam_word(row - 1, 2);
        self.set_oam_word(row, 0, ((a ^ c) & (b ^ c)) ^ c);
        self.copy_oam_row_suffix_from_previous(row);
    }

    fn apply_oam_bug_read(&mut self, row: usize) {
        if row == 0 || row >= 20 {
            return;
        }
        let a = self.oam_word(row, 0);
        let b = self.oam_word(row - 1, 0);
        let c = self.oam_word(row - 1, 2);
        self.set_oam_word(row, 0, b | (a & c));
        self.copy_oam_row_suffix_from_previous(row);
    }

    fn apply_oam_bug_read_during_idu(&mut self, row: usize) {
        // Combined read+IDU corruption has an additional pre-step on most rows.
        if (4..=18).contains(&row) {
            let a = self.oam_word(row - 2, 0);
            let b = self.oam_word(row - 1, 0);
            let c = self.oam_word(row, 0);
            let d = self.oam_word(row - 1, 2);
            let mixed = (b & (a | c | d)) | (a & c & d);
            self.set_oam_word(row - 1, 0, mixed);
            self.copy_oam_full_row(row, row - 1);
            self.copy_oam_full_row(row - 2, row - 1);
        }

        self.apply_oam_bug_read(row);
    }

    fn trigger_oam_bug_on_read_access(&mut self, addr: u16) {
        if self.is_cgb() {
            return;
        }
        if !(0xFE00..=0xFEFF).contains(&addr) {
            return;
        }
        if self.oam_bug_read_idu_pending_addr == Some(addr) {
            self.oam_bug_read_idu_pending_addr = None;
            if let Some(row) = self.oam_bug_row() {
                self.apply_oam_bug_read_during_idu(row);
            }
            return;
        }
        if let Some(row) = self.oam_bug_row() {
            self.apply_oam_bug_read(row);
        }
    }

    fn trigger_oam_bug_on_write_access(&mut self, addr: u16) {
        if self.is_cgb() {
            return;
        }
        if !(0xFE00..=0xFEFF).contains(&addr) {
            return;
        }
        if let Some(row) = self.oam_bug_row() {
            self.apply_oam_bug_write(row);
        }
    }

    pub fn trigger_oam_bug_idu_write(&mut self, idu_addr: u16) {
        if self.is_cgb() {
            return;
        }
        if !(0xFE00..=0xFEFF).contains(&idu_addr) {
            return;
        }
        if let Some(row) = self.oam_bug_row() {
            self.apply_oam_bug_write(row);
        }
    }

    pub fn schedule_oam_bug_idu_read(&mut self, idu_addr: u16) {
        if self.is_cgb() || !(0xFE00..=0xFEFF).contains(&idu_addr) {
            self.oam_bug_read_idu_pending_addr = None;
            return;
        }
        self.oam_bug_read_idu_pending_addr = Some(idu_addr);
    }

    /// Returns true if the CGB speed-switch handshake was performed.
    pub fn try_cgb_speed_switch(&mut self) -> bool {
        if !self.is_cgb() || !self.cgb_speed_switch_prepare {
            return false;
        }

        self.cgb_speed_switch_prepare = false;
        self.cgb_double_speed = !self.cgb_double_speed;
        true
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        if self
            .oam_bug_read_idu_pending_addr
            .is_some_and(|pending| pending != addr)
        {
            self.oam_bug_read_idu_pending_addr = None;
        }
        if self.oam_dma.blocks_cpu_addr(addr) {
            return 0xFF;
        }
        self.trigger_oam_bug_on_read_access(addr);
        if self.cpu_access_blocked_by_ppu(addr) {
            return 0xFF;
        }
        self.read8_direct(addr)
    }

    fn read8_direct(&mut self, addr: u16) -> u8 {
        match addr {
            // ROM: 0x0000..=0x7FFF
            0x0000..=0x7FFF => self.cart.mbc.read_rom(&self.cart.rom, addr),

            // VRAM: 0x8000..=0x9FFF
            0x8000..=0x9FFF => {
                let bank = if self.is_cgb() {
                    self.cgb_vram_bank as usize
                } else {
                    0
                };
                let offset = (addr - 0x8000) as usize;
                self.vram[bank * 0x2000 + offset]
            }

            // Cartridge RAM: 0xA000..=0xBFFF
            0xA000..=0xBFFF => self.cart.mbc.read_ram(&self.cart.ram, addr),

            // WRAM: 0xC000..=0xDFFF
            0xC000..=0xDFFF => self.read_wram(addr),

            // Echo WRAM: 0xE000..=0xFDFF (mirrors 0xC000..=0xDFFF)
            0xE000..=0xFDFF => self.read_wram(addr),

            // OAM: 0xFE00..=0xFE9F
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],

            // Unusable: 0xFEA0..=0xFEFF
            0xFEA0..=0xFEFF => 0xFF,

            // IO Registers: 0xFF00..=0xFF7F
            0xFF00..=0xFF7F => match addr {
                0xFF00 => self.input.read_joyp(),
                0xFF04 => self.timer.read_div(),
                0xFF05 => self.timer.read_tima(),
                0xFF06 => self.timer.read_tma(),
                0xFF07 => self.timer.read_tac(),
                0xFF0F => self.iflag | 0xE0,
                0xFF10..=0xFF3F => self.apu.read_register(addr),
                0xFF51 => self.read_hdma1(),
                0xFF52 => self.read_hdma2(),
                0xFF53 => self.read_hdma3(),
                0xFF54 => self.read_hdma4(),
                0xFF55 => self.read_hdma5(),
                0xFF68 => {
                    if self.is_cgb() {
                        self.ppu.read_bgpi()
                    } else {
                        0xFF
                    }
                }
                0xFF69 => {
                    if self.is_cgb() {
                        self.ppu.read_bgpd()
                    } else {
                        0xFF
                    }
                }
                0xFF6A => {
                    if self.is_cgb() {
                        self.ppu.read_obpi()
                    } else {
                        0xFF
                    }
                }
                0xFF6B => {
                    if self.is_cgb() {
                        self.ppu.read_obpd()
                    } else {
                        0xFF
                    }
                }
                0xFF4F => self.read_vbk(),
                0xFF70 => self.read_svbk(),
                0xFF4D => self.read_key1(),
                _ => self.io[(addr - 0xFF00) as usize],
            },

            // HRAM: 0xFF80..=0xFFFE
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // IE Register: 0xFFFF
            0xFFFF => self.ie,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
        if self.oam_dma.blocks_cpu_addr(addr) {
            return;
        }
        self.trigger_oam_bug_on_write_access(addr);
        if self.cpu_access_blocked_by_ppu(addr) {
            return;
        }
        self.write8_direct(addr, val);
    }

    fn write8_direct(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM: 0x0000..=0x7FFF (writes go to MBC control)
            0x0000..=0x7FFF => self.cart.mbc.write_rom(addr, val),

            // VRAM: 0x8000..=0x9FFF
            0x8000..=0x9FFF => {
                let bank = if self.is_cgb() {
                    self.cgb_vram_bank as usize
                } else {
                    0
                };
                let offset = (addr - 0x8000) as usize;
                self.vram[bank * 0x2000 + offset] = val;
            }

            // Cartridge RAM: 0xA000..=0xBFFF
            0xA000..=0xBFFF => self.cart.mbc.write_ram(&mut self.cart.ram, addr, val),

            // WRAM: 0xC000..=0xDFFF
            0xC000..=0xDFFF => self.write_wram(addr, val),

            // Echo WRAM: 0xE000..=0xFDFF (mirrors 0xC000..=0xDFFF)
            0xE000..=0xFDFF => self.write_wram(addr, val),

            // OAM: 0xFE00..=0xFE9F
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,

            // Unusable: 0xFEA0..=0xFEFF
            0xFEA0..=0xFEFF => {}

            // IO Registers: 0xFF00..=0xFF7F
            0xFF00..=0xFF7F => {
                let idx = (addr - 0xFF00) as usize;
                match addr {
                    0xFF00 => self.input.write_joyp(val),
                    0xFF04 => self.timer.write_div(&mut self.iflag),
                    0xFF05 => self.timer.write_tima(val),
                    0xFF06 => self.timer.write_tma(val),
                    0xFF07 => self.timer.write_tac(val, &mut self.iflag),
                    0xFF0F => self.iflag = val & 0x1F,
                    // APU register accesses take an M-cycle. Some APU behaviors (notably NR52
                    // power-up re-phasing on CGB) are sensitive to the global DIV phase.
                    // Use the DIV value at the *end* of the access (+4 cycles).
                    0xFF10..=0xFF3F => self.apu.write_register(addr, val, self.timer.raw_counter()),
                    0xFF4F => self.write_vbk(val),
                    0xFF4D => self.write_key1(val),
                    0xFF70 => self.write_svbk(val),
                    0xFF51 => self.write_hdma1(val),
                    0xFF52 => self.write_hdma2(val),
                    0xFF53 => self.write_hdma3(val),
                    0xFF54 => self.write_hdma4(val),
                    0xFF55 => self.start_hdma_transfer(val),
                    0xFF68 => {
                        if self.is_cgb() {
                            self.ppu.write_bgpi(val);
                        }
                    }
                    0xFF69 => {
                        if self.is_cgb() {
                            self.ppu.write_bgpd(val);
                        }
                    }
                    0xFF6A => {
                        if self.is_cgb() {
                            self.ppu.write_obpi(val);
                        }
                    }
                    0xFF6B => {
                        if self.is_cgb() {
                            self.ppu.write_obpd(val);
                        }
                    }
                    0xFF02 => {
                        self.io[idx] = val;
                        // Common test ROM convention: write a byte to SB (0xFF01), then write 0x81
                        // to SC (0xFF02) to start a serial transfer.
                        if (val & 0x80) != 0 {
                            self.serial.start_transfer(self.io[0x01], &mut self.io[idx]);
                        } else {
                            self.serial.stop_transfer(&mut self.io[idx]);
                        }
                    }
                    0xFF41 => self.io[idx] = (self.io[idx] & 0x07) | (val & 0x78),
                    0xFF44 => {
                        self.io[idx] = 0;
                        self.ppu.reset_ly();
                    }
                    0xFF46 => {
                        self.io[idx] = val;
                        self.oam_dma.start(val);
                    }
                    _ => self.io[idx] = val,
                }
            }

            // HRAM: 0xFF80..=0xFFFE
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,

            // IE Register: 0xFFFF
            0xFFFF => self.ie = val,
        }
    }

    pub fn set_joypad_button(&mut self, button: crate::input::Button, pressed: bool) {
        self.input.set_button(button, pressed, &mut self.iflag);
    }

    pub fn tick(&mut self, cycles: u32) {
        // The emulator's CPU executes in 4-cycle M-cycles. On CGB, the CPU can run in
        // double-speed mode (KEY1). In that mode, *only the CPU* runs at 2x frequency;
        // PPU/APU/timer/serial remain tied to the base 4_194_304 Hz clock.
        //
        // We model this by interpreting `cycles` as CPU cycles, and converting to
        // base ("system") cycles before ticking the rest of the bus.
        let cycles = if self.is_cgb() && self.cgb_double_speed {
            debug_assert_eq!(cycles % 2, 0, "double-speed tick requires even cycle count");
            cycles / 2
        } else {
            cycles
        };

        self.cart.mbc.tick(cycles);
        self.timer.tick(cycles, &mut self.iflag);
        self.tick_oam_dma(cycles);
        let vram0: &[u8; 0x2000] = self.vram[..0x2000]
            .try_into()
            .expect("slice length for vram0 is fixed");
        let vram1: &[u8; 0x2000] = self.vram[0x2000..]
            .try_into()
            .expect("slice length for vram1 is fixed");
        let cgb_mode = self.is_cgb();
        self.ppu.tick_with_vram_banks(
            cycles,
            vram0,
            Some(vram1),
            &self.oam,
            &mut self.io,
            &mut self.iflag,
            cgb_mode,
        );
        self.tick_hdma();
        self.apu.tick(cycles);
        self.serial
            .tick(cycles, &mut self.iflag, &mut self.io[0x02]);
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), crate::cartridge::SaveError> {
        self.cart.save_to_path(path)
    }

    pub fn load_from_path(&mut self, path: &Path) -> Result<(), crate::cartridge::SaveError> {
        self.cart.load_from_path(path)
    }

    fn tick_oam_dma(&mut self, cycles: u32) {
        self.oam_dma.add_cycles(cycles);
        while let Some((src, dst)) = self.oam_dma.pop_transfer() {
            let v = self.read8_direct(src);
            self.oam[dst] = v;
        }
    }
}
