use crate::bus::Bus;

use super::cpu::{Cpu, Flag, R8};

#[inline]
fn decode_r8(idx: u8) -> R8 {
    match idx & 0x07 {
        0x0 => R8::B,
        0x1 => R8::C,
        0x2 => R8::D,
        0x3 => R8::E,
        0x4 => R8::H,
        0x5 => R8::L,
        0x6 => R8::HlInd,
        0x7 => R8::A,
        _ => unreachable!(),
    }
}

#[inline]
fn cycles_for_target(r: R8) -> u32 {
    if matches!(r, R8::HlInd) {
        16
    } else {
        8
    }
}

#[inline]
fn bit_cycles_for_target(r: R8) -> u32 {
    if matches!(r, R8::HlInd) {
        12
    } else {
        8
    }
}

// CB-prefixed (0xCBxx) instruction implementations
pub fn exec(cpu: &mut Cpu, bus: &mut Bus, opcode: u8) -> u32 {
    let r = decode_r8(opcode);

    match opcode {
        0x00..=0x3F => {
            let op = (opcode >> 3) & 0x07;
            let v = cpu.read_r8(bus, r);
            let carry_in = cpu.flag(Flag::C) as u8;

            let (res, carry_out) = match op {
                // RLC r
                0x0 => {
                    let c = (v & 0x80) != 0;
                    (v.rotate_left(1), c)
                }
                // RRC r
                0x1 => {
                    let c = (v & 0x01) != 0;
                    (v.rotate_right(1), c)
                }
                // RL r
                0x2 => {
                    let c = (v & 0x80) != 0;
                    ((v << 1) | carry_in, c)
                }
                // RR r
                0x3 => {
                    let c = (v & 0x01) != 0;
                    ((v >> 1) | (carry_in << 7), c)
                }
                // SLA r
                0x4 => {
                    let c = (v & 0x80) != 0;
                    (v << 1, c)
                }
                // SRA r
                0x5 => {
                    let c = (v & 0x01) != 0;
                    ((v >> 1) | (v & 0x80), c)
                }
                // SWAP r
                0x6 => (v.rotate_right(4), false),
                // SRL r
                0x7 => {
                    let c = (v & 0x01) != 0;
                    (v >> 1, c)
                }
                _ => unreachable!(),
            };

            cpu.write_r8(bus, r, res);

            cpu.set_flag(Flag::Z, res == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, carry_out);

            cycles_for_target(r)
        }
        0x40..=0x7F => {
            // BIT b,r
            let bit = (opcode >> 3) & 0x07;
            let v = cpu.read_r8(bus, r);
            cpu.set_flag(Flag::Z, (v & (1 << bit)) == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, true);
            bit_cycles_for_target(r)
        }
        0x80..=0xBF => {
            // RES b,r
            let bit = (opcode >> 3) & 0x07;
            let v = cpu.read_r8(bus, r);
            let res = v & !(1 << bit);
            cpu.write_r8(bus, r, res);
            cycles_for_target(r)
        }
        0xC0..=0xFF => {
            // SET b,r
            let bit = (opcode >> 3) & 0x07;
            let v = cpu.read_r8(bus, r);
            let res = v | (1 << bit);
            cpu.write_r8(bus, r, res);
            cycles_for_target(r)
        }
    }
}
