//! Interrupt helper types.

/// Interrupt bits and vectors, in CPU priority order.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Interrupt {
    VBlank = 0,
    LcdStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

impl Interrupt {
    #[inline]
    pub const fn bit(self) -> u8 {
        1 << (self as u8)
    }

    #[inline]
    pub const fn vector(self) -> u16 {
        match self {
            Self::VBlank => 0x0040,
            Self::LcdStat => 0x0048,
            Self::Timer => 0x0050,
            Self::Serial => 0x0058,
            Self::Joypad => 0x0060,
        }
    }

    #[inline]
    pub fn from_pending_mask(pending: u8) -> Option<Self> {
        if pending == 0 {
            return None;
        }
        match pending.trailing_zeros() as u8 {
            0 => Some(Self::VBlank),
            1 => Some(Self::LcdStat),
            2 => Some(Self::Timer),
            3 => Some(Self::Serial),
            4 => Some(Self::Joypad),
            _ => None,
        }
    }
}

#[inline]
pub const fn pending_mask(ie: u8, iflag: u8) -> u8 {
    ie & iflag & 0x1F
}
