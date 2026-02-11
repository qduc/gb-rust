use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::dma;
use crate::input::Joypad;
use crate::ppu::Ppu;
use crate::serial::Serial;
use crate::timer::Timer;
use std::path::Path;

pub struct Bus {
    pub cart: Cartridge,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub input: Joypad,
    pub serial: Serial,
    pub wram: [u8; 0x2000],
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub io: [u8; 0x80],
    pub hram: [u8; 0x7F],
    pub ie: u8,
    pub iflag: u8,
    pub oam_dma: dma::OamDma,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cart,
            ppu: Ppu::new(),
            apu: Apu::new(),
            timer: Timer::new(),
            input: Joypad::new(),
            serial: Serial::new(),
            wram: [0; 0x2000],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
            iflag: 0,
            oam_dma: dma::OamDma::default(),
        }
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
        if self.oam_dma.blocks_cpu_addr(addr) {
            return 0xFF;
        }
        self.read8_direct(addr)
    }

    fn read8_direct(&mut self, addr: u16) -> u8 {
        match addr {
            // ROM: 0x0000..=0x7FFF
            0x0000..=0x7FFF => self.cart.mbc.read_rom(&self.cart.rom, addr),

            // VRAM: 0x8000..=0x9FFF
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize],

            // Cartridge RAM: 0xA000..=0xBFFF
            0xA000..=0xBFFF => self.cart.mbc.read_ram(&self.cart.ram, addr),

            // WRAM: 0xC000..=0xDFFF
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize],

            // Echo WRAM: 0xE000..=0xFDFF (mirrors 0xC000..=0xDFFF)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize],

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
        self.write8_direct(addr, val);
    }

    fn write8_direct(&mut self, addr: u16, val: u8) {
        match addr {
            // ROM: 0x0000..=0x7FFF (writes go to MBC control)
            0x0000..=0x7FFF => self.cart.mbc.write_rom(addr, val),

            // VRAM: 0x8000..=0x9FFF
            0x8000..=0x9FFF => self.vram[(addr - 0x8000) as usize] = val,

            // Cartridge RAM: 0xA000..=0xBFFF
            0xA000..=0xBFFF => self.cart.mbc.write_ram(&mut self.cart.ram, addr, val),

            // WRAM: 0xC000..=0xDFFF
            0xC000..=0xDFFF => self.wram[(addr - 0xC000) as usize] = val,

            // Echo WRAM: 0xE000..=0xFDFF (mirrors 0xC000..=0xDFFF)
            0xE000..=0xFDFF => self.wram[(addr - 0xE000) as usize] = val,

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
                    0xFF10..=0xFF3F => self.apu.write_register(addr, val),
                    0xFF02 => {
                        self.io[idx] = val;
                        // Common test ROM convention: write a byte to SB (0xFF01), then write 0x81
                        // to SC (0xFF02) to "transfer" it out via the serial port.
                        if (val & 0x80) != 0 {
                            self.serial.on_transfer(self.io[0x01]);
                            self.io[idx] = val & 0x7F; // clear transfer-start bit
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
        self.cart.mbc.tick(cycles);
        self.timer.tick(cycles, &mut self.iflag);
        self.tick_oam_dma(cycles);
        self.ppu
            .tick(cycles, &self.vram, &self.oam, &mut self.io, &mut self.iflag);
        self.apu.tick(cycles);
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
