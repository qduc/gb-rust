use crate::bus::Bus;
use crate::interrupt::{pending_mask, Interrupt};

use super::{cb_ops, ops};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum R8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    /// Memory at address in HL.
    HlInd,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

impl Flag {
    const fn mask(self) -> u8 {
        match self {
            Self::Z => 0x80,
            Self::N => 0x40,
            Self::H => 0x20,
            Self::C => 0x10,
        }
    }
}

pub struct Cpu {
    // 8-bit registers
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // 16-bit registers
    pub sp: u16,
    pub pc: u16,

    // CPU state
    pub ime: bool,
    pub halted: bool,
    /// Set by EI; IME becomes true after the following instruction completes.
    pub ei_pending: bool,
    /// HALT bug latch: next opcode fetch reads at PC without incrementing it.
    pub halt_bug: bool,
    step_cycles: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            ime: false,
            halted: false,
            ei_pending: false,
            halt_bug: false,
            step_cycles: 0,
        }
    }

    #[inline]
    fn service_interrupt(&mut self, bus: &mut Bus, pending: u8) -> u32 {
        let intr =
            Interrupt::from_pending_mask(pending).expect("service_interrupt called with pending=0");

        bus.iflag &= !intr.bit();
        self.ime = false;
        self.halted = false;

        let pc = self.pc;
        self.push16(bus, pc);
        self.pc = intr.vector();

        self.finish_step(bus, 20)
    }

    #[inline]
    fn tick_mcycle(&mut self, bus: &mut Bus) {
        bus.tick(4);
        self.step_cycles += 4;
    }

    #[inline]
    fn tick_idle(&mut self, bus: &mut Bus, cycles: u32) {
        debug_assert_eq!(cycles % 4, 0);
        bus.tick(cycles);
        self.step_cycles += cycles;
    }

    #[inline]
    fn finish_step(&mut self, bus: &mut Bus, target_cycles: u32) -> u32 {
        debug_assert_eq!(target_cycles % 4, 0);
        if self.step_cycles < target_cycles {
            self.tick_idle(bus, target_cycles - self.step_cycles);
        }
        debug_assert_eq!(self.step_cycles, target_cycles);
        target_cycles
    }

    #[inline]
    pub fn read8(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        let v = bus.read8(addr);
        self.tick_mcycle(bus);
        v
    }

    #[inline]
    pub fn write8(&mut self, bus: &mut Bus, addr: u16, val: u8) {
        bus.write8(addr, val);
        self.tick_mcycle(bus);
    }

    #[inline]
    pub fn fetch8(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.pc;
        let v = self.read8(bus, addr);
        if self.halt_bug {
            self.halt_bug = false;
        } else {
            self.pc = self.pc.wrapping_add(1);
        }
        v
    }

    #[inline]
    pub fn fetch16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch8(bus);
        let hi = self.fetch8(bus);
        u16::from_le_bytes([lo, hi])
    }

    #[inline]
    pub fn af(&self) -> u16 {
        u16::from_be_bytes([self.a, self.f & 0xF0])
    }

    #[inline]
    pub fn set_af(&mut self, v: u16) {
        let [a, f] = v.to_be_bytes();
        self.a = a;
        self.f = f & 0xF0;
    }

    #[inline]
    pub fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    #[inline]
    pub fn set_bc(&mut self, v: u16) {
        let [b, c] = v.to_be_bytes();
        self.b = b;
        self.c = c;
    }

    #[inline]
    pub fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    #[inline]
    pub fn set_de(&mut self, v: u16) {
        let [d, e] = v.to_be_bytes();
        self.d = d;
        self.e = e;
    }

    #[inline]
    pub fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    #[inline]
    pub fn set_hl(&mut self, v: u16) {
        let [h, l] = v.to_be_bytes();
        self.h = h;
        self.l = l;
    }

    #[inline]
    pub fn read_r8(&mut self, bus: &mut Bus, r: R8) -> u8 {
        match r {
            R8::A => self.a,
            R8::F => self.f & 0xF0,
            R8::B => self.b,
            R8::C => self.c,
            R8::D => self.d,
            R8::E => self.e,
            R8::H => self.h,
            R8::L => self.l,
            R8::HlInd => self.read8(bus, self.hl()),
        }
    }

    #[inline]
    pub fn write_r8(&mut self, bus: &mut Bus, r: R8, v: u8) {
        match r {
            R8::A => self.a = v,
            R8::F => self.f = v & 0xF0,
            R8::B => self.b = v,
            R8::C => self.c = v,
            R8::D => self.d = v,
            R8::E => self.e = v,
            R8::H => self.h = v,
            R8::L => self.l = v,
            R8::HlInd => {
                let addr = self.hl();
                self.write8(bus, addr, v);
            }
        }
    }

    #[inline]
    pub fn push16(&mut self, bus: &mut Bus, v: u16) {
        let [hi, lo] = v.to_be_bytes();
        self.sp = self.sp.wrapping_sub(1);
        self.write8(bus, self.sp, hi);
        self.sp = self.sp.wrapping_sub(1);
        self.write8(bus, self.sp, lo);
    }

    #[inline]
    pub fn pop16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.read8(bus, self.sp);
        self.sp = self.sp.wrapping_add(1);
        let hi = self.read8(bus, self.sp);
        self.sp = self.sp.wrapping_add(1);
        u16::from_be_bytes([hi, lo])
    }

    #[inline]
    pub fn flag(&self, flag: Flag) -> bool {
        (self.f & flag.mask()) != 0
    }

    #[inline]
    pub fn set_flag(&mut self, flag: Flag, on: bool) {
        if on {
            self.f |= flag.mask();
        } else {
            self.f &= !flag.mask();
        }
        self.f &= 0xF0;
    }

    pub fn step(&mut self, bus: &mut Bus) -> u32 {
        self.step_cycles = 0;

        let pending = pending_mask(bus.ie, bus.iflag);

        if self.halted {
            if pending == 0 {
                self.tick_idle(bus, 4);
                return 4;
            }

            self.halted = false;
            if self.ime {
                return self.service_interrupt(bus, pending);
            }
            self.halt_bug = true;
        } else if self.ime && pending != 0 {
            return self.service_interrupt(bus, pending);
        }

        // EI delay: IME is enabled after the *following* instruction completes.
        let enable_ime_after = self.ei_pending;
        self.ei_pending = false;

        let opcode = self.fetch8(bus);
        let cycles = if opcode == 0xCB {
            let cb = self.fetch8(bus);
            cb_ops::exec(self, bus, cb)
        } else {
            ops::exec(self, bus, opcode)
        };

        if enable_ime_after && opcode != 0xF3 {
            self.ime = true;
        }

        self.finish_step(bus, cycles)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}
