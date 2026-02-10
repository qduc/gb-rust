use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::dma;
use crate::input::Joypad;
use crate::ppu::Ppu;
use crate::timer::Timer;

pub struct Bus {
    pub cart: Cartridge,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub input: Joypad,
    pub wram: [u8; 0x2000],
    pub vram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub io: [u8; 0x80],
    pub hram: [u8; 0x7F],
    pub ie: u8,
    pub iflag: u8,
}

impl Bus {
    pub fn new(cart: Cartridge) -> Self {
        Self {
            cart,
            ppu: Ppu::new(),
            apu: Apu::new(),
            timer: Timer::new(),
            input: Joypad::new(),
            wram: [0; 0x2000],
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: 0,
            iflag: 0,
        }
    }

    pub fn read8(&mut self, addr: u16) -> u8 {
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
                0xFF04 => self.timer.read_div(),
                0xFF05 => self.timer.read_tima(),
                0xFF06 => self.timer.read_tma(),
                0xFF07 => self.timer.read_tac(),
                0xFF0F => self.iflag | 0xE0,
                _ => self.io[(addr - 0xFF00) as usize],
            },

            // HRAM: 0xFF80..=0xFFFE
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],

            // IE Register: 0xFFFF
            0xFFFF => self.ie,
        }
    }

    pub fn write8(&mut self, addr: u16, val: u8) {
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
                    0xFF04 => self.timer.write_div(&mut self.iflag),
                    0xFF05 => self.timer.write_tima(val),
                    0xFF06 => self.timer.write_tma(val),
                    0xFF07 => self.timer.write_tac(val, &mut self.iflag),
                    0xFF0F => self.iflag = val & 0x1F,
                    0xFF41 => self.io[idx] = (self.io[idx] & 0x07) | (val & 0x78),
                    0xFF44 => {
                        self.io[idx] = 0;
                        self.ppu.reset_ly();
                    }
                    0xFF46 => {
                        self.io[idx] = val;
                        dma::oam_dma(self, val);
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

    pub fn tick(&mut self, cycles: u32) {
        self.timer.tick(cycles, &mut self.iflag);
        self.ppu.tick(cycles, &mut self.io, &mut self.iflag);
        self.apu.tick(cycles);
    }
}
