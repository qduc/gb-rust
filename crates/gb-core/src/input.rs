use crate::interrupt::Interrupt;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Button {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Select,
    Start,
}

impl Button {
    const fn mask(self) -> u8 {
        match self {
            Self::Right => 1 << 0,
            Self::Left => 1 << 1,
            Self::Up => 1 << 2,
            Self::Down => 1 << 3,
            Self::A => 1 << 4,
            Self::B => 1 << 5,
            Self::Select => 1 << 6,
            Self::Start => 1 << 7,
        }
    }
}

/// Joypad (JOYP/P1) register + button state.
///
/// - 0xFF00 bits 4-5 are selection lines (active low)
/// - bits 0-3 are button state (active low)
/// - bits 6-7 read as 1
#[derive(Serialize, Deserialize)]
pub struct Joypad {
    /// Raw selection bits (bits 4-5) written by CPU (active low).
    select: u8,
    /// Button state bitmask; 1 = pressed.
    state: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            select: 0x30,
            state: 0,
        }
    }

    #[inline]
    pub fn read_joyp(&self) -> u8 {
        let select_buttons = (self.select & 0x20) == 0;
        let select_directions = (self.select & 0x10) == 0;

        let dir_nibble = if select_directions {
            (!(self.state & 0x0F)) & 0x0F
        } else {
            0x0F
        };

        let btn_nibble = if select_buttons {
            (!((self.state >> 4) & 0x0F)) & 0x0F
        } else {
            0x0F
        };

        let nibble = dir_nibble & btn_nibble;
        0xC0 | (self.select & 0x30) | nibble
    }

    #[inline]
    pub fn write_joyp(&mut self, val: u8) {
        self.select = val & 0x30;
    }

    #[inline]
    pub fn set_button(&mut self, button: Button, pressed: bool, iflag: &mut u8) {
        let mask = button.mask();
        let was_pressed = (self.state & mask) != 0;

        if pressed {
            self.state |= mask;
            if !was_pressed {
                *iflag |= Interrupt::Joypad.bit();
            }
        } else {
            self.state &= !mask;
        }
    }
}

impl Default for Joypad {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joypad_read_reflects_direction_keys_when_selected() {
        let mut jp = Joypad::new();
        let mut iflag = 0u8;

        // Select directions (P14=0, P15=1).
        jp.write_joyp(0x20);
        jp.set_button(Button::Right, true, &mut iflag);

        assert_eq!(iflag & Interrupt::Joypad.bit(), Interrupt::Joypad.bit());
        assert_eq!(jp.read_joyp() & 0x0F, 0x0E); // right pressed => bit0 low
    }

    #[test]
    fn joypad_read_reflects_buttons_when_selected() {
        let mut jp = Joypad::new();
        let mut iflag = 0u8;

        // Select buttons (P15=0, P14=1).
        jp.write_joyp(0x10);
        jp.set_button(Button::A, true, &mut iflag);

        assert_eq!(jp.read_joyp() & 0x0F, 0x0E); // A pressed => bit0 low
    }

    #[test]
    fn joypad_unselected_group_reads_high() {
        let mut jp = Joypad::new();
        let mut iflag = 0u8;

        // Select buttons only.
        jp.write_joyp(0x10);
        jp.set_button(Button::Right, true, &mut iflag);

        // Directions are unselected => low nibble stays 0x0F.
        assert_eq!(jp.read_joyp() & 0x0F, 0x0F);
    }
}
