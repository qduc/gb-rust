use crate::interrupt::Interrupt;

/// DMG timer registers:
/// - DIV  (FF04) = upper 8 bits of an internal 16-bit counter
/// - TIMA (FF05)
/// - TMA  (FF06)
/// - TAC  (FF07)
pub struct Timer {
    counter: u16,
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }

    #[inline]
    fn input_bit(counter: u16, tac: u8) -> bool {
        if (tac & 0x04) == 0 {
            return false;
        }

        let bit = match tac & 0x03 {
            0x00 => 9, // 4096 Hz   => 1024 cycles
            0x01 => 3, // 262144 Hz => 16 cycles
            0x02 => 5, // 65536 Hz  => 64 cycles
            0x03 => 7, // 16384 Hz  => 256 cycles
            _ => unreachable!(),
        };

        (counter & (1 << bit)) != 0
    }

    #[inline]
    fn inc_tima(&mut self, iflag: &mut u8) {
        let (v, overflow) = self.tima.overflowing_add(1);
        if overflow {
            self.tima = self.tma;
            *iflag |= Interrupt::Timer.bit();
        } else {
            self.tima = v;
        }
    }

    #[inline]
    pub fn read_div(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    #[inline]
    pub fn write_div(&mut self, iflag: &mut u8) {
        // DIV reset can create a falling edge on the timer input.
        let old = Self::input_bit(self.counter, self.tac);
        self.counter = 0;
        let new = Self::input_bit(self.counter, self.tac);
        if old && !new {
            self.inc_tima(iflag);
        }
    }

    #[inline]
    pub fn read_tima(&self) -> u8 {
        self.tima
    }

    #[inline]
    pub fn write_tima(&mut self, val: u8) {
        self.tima = val;
    }

    #[inline]
    pub fn read_tma(&self) -> u8 {
        self.tma
    }

    #[inline]
    pub fn write_tma(&mut self, val: u8) {
        self.tma = val;
    }

    #[inline]
    pub fn read_tac(&self) -> u8 {
        self.tac | 0xF8
    }

    #[inline]
    pub fn write_tac(&mut self, val: u8, iflag: &mut u8) {
        // TAC change can create a falling edge on the timer input.
        let old = Self::input_bit(self.counter, self.tac);
        self.tac = val & 0x07;
        let new = Self::input_bit(self.counter, self.tac);
        if old && !new {
            self.inc_tima(iflag);
        }
    }

    pub fn tick(&mut self, cycles: u32, iflag: &mut u8) {
        for _ in 0..(cycles as usize) {
            let old = Self::input_bit(self.counter, self.tac);
            self.counter = self.counter.wrapping_add(1);
            let new = Self::input_bit(self.counter, self.tac);
            if old && !new {
                self.inc_tima(iflag);
            }
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
