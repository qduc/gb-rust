use crate::bus::Bus;
use crate::interrupt::pending_mask;

use super::cpu::{Flag, R8};
use super::Cpu;

fn r8_from_code(code: u8) -> R8 {
    match code & 0x07 {
        0 => R8::B,
        1 => R8::C,
        2 => R8::D,
        3 => R8::E,
        4 => R8::H,
        5 => R8::L,
        6 => R8::HlInd,
        _ => R8::A,
    }
}

fn alu_add(cpu: &mut Cpu, a: u8, b: u8, carry_in: u8) -> u8 {
    let sum = a as u16 + b as u16 + carry_in as u16;
    let res = sum as u8;
    cpu.set_flag(Flag::Z, res == 0);
    cpu.set_flag(Flag::N, false);
    cpu.set_flag(Flag::H, ((a & 0x0F) + (b & 0x0F) + carry_in) > 0x0F);
    cpu.set_flag(Flag::C, sum > 0xFF);
    res
}

fn alu_sub(cpu: &mut Cpu, a: u8, b: u8, carry_in: u8) -> u8 {
    let sub = a as i16 - b as i16 - carry_in as i16;
    let res = sub as u8;
    cpu.set_flag(Flag::Z, res == 0);
    cpu.set_flag(Flag::N, true);
    cpu.set_flag(Flag::H, (a & 0x0F) < ((b & 0x0F) + carry_in));
    cpu.set_flag(Flag::C, (a as u16) < (b as u16 + carry_in as u16));
    res
}

fn cond(cpu: &Cpu, opcode: u8) -> bool {
    match opcode {
        0x20 | 0xC0 | 0xC2 | 0xC4 => !cpu.flag(Flag::Z),
        0x28 | 0xC8 | 0xCA | 0xCC => cpu.flag(Flag::Z),
        0x30 | 0xD0 | 0xD2 | 0xD4 => !cpu.flag(Flag::C),
        0x38 | 0xD8 | 0xDA | 0xDC => cpu.flag(Flag::C),
        _ => true,
    }
}

fn jr(cpu: &mut Cpu, off: i8) {
    let pc = cpu.pc as i32 + off as i32;
    cpu.pc = pc as u16;
}

fn daa(cpu: &mut Cpu) {
    let mut a = cpu.a;
    let mut adjust = 0u8;
    let mut c = cpu.flag(Flag::C);

    if !cpu.flag(Flag::N) {
        if cpu.flag(Flag::H) || (a & 0x0F) > 0x09 {
            adjust |= 0x06;
        }
        if c || a > 0x99 {
            adjust |= 0x60;
            c = true;
        }
        a = a.wrapping_add(adjust);
    } else {
        if cpu.flag(Flag::H) {
            adjust |= 0x06;
        }
        if c {
            adjust |= 0x60;
        }
        a = a.wrapping_sub(adjust);
    }

    cpu.a = a;
    cpu.set_flag(Flag::Z, cpu.a == 0);
    cpu.set_flag(Flag::H, false);
    cpu.set_flag(Flag::C, c);
}

// Non-CB instruction implementations
pub fn exec(cpu: &mut Cpu, bus: &mut Bus, opcode: u8) -> u32 {
    match opcode {
        0x00 => 4, // NOP

        0x10 => {
            // STOP; consume the following padding byte.
            let _ = cpu.fetch8(bus);

            // On CGB, STOP is also used for the KEY1 speed-switch handshake.
            cpu.halted = !bus.try_cgb_speed_switch();

            8
        }

        // 16-bit loads
        0x01 => {
            let v = cpu.fetch16(bus);
            cpu.set_bc(v);
            12
        }
        0x11 => {
            let v = cpu.fetch16(bus);
            cpu.set_de(v);
            12
        }
        0x21 => {
            let v = cpu.fetch16(bus);
            cpu.set_hl(v);
            12
        }
        0x31 => {
            cpu.sp = cpu.fetch16(bus);
            12
        }

        0x08 => {
            let addr = cpu.fetch16(bus);
            let [lo, hi] = cpu.sp.to_le_bytes();
            cpu.write8(bus, addr, lo);
            cpu.write8(bus, addr.wrapping_add(1), hi);
            20
        }

        // LD (rr),A / LD A,(rr)
        0x02 => {
            let addr = cpu.bc();
            cpu.write8(bus, addr, cpu.a);
            8
        }
        0x0A => {
            let addr = cpu.bc();
            cpu.a = cpu.read8(bus, addr);
            8
        }
        0x12 => {
            let addr = cpu.de();
            cpu.write8(bus, addr, cpu.a);
            8
        }
        0x1A => {
            let addr = cpu.de();
            cpu.a = cpu.read8(bus, addr);
            8
        }

        // LD (HL+/-),A / LD A,(HL+/-)
        0x22 => {
            let addr = cpu.hl();
            cpu.write8(bus, addr, cpu.a);
            cpu.set_hl(addr.wrapping_add(1));
            8
        }
        0x2A => {
            let addr = cpu.hl();
            cpu.a = cpu.read8(bus, addr);
            cpu.set_hl(addr.wrapping_add(1));
            8
        }
        0x32 => {
            let addr = cpu.hl();
            cpu.write8(bus, addr, cpu.a);
            cpu.set_hl(addr.wrapping_sub(1));
            8
        }
        0x3A => {
            let addr = cpu.hl();
            cpu.a = cpu.read8(bus, addr);
            cpu.set_hl(addr.wrapping_sub(1));
            8
        }

        // LD (a16),A / LD A,(a16)
        0xEA => {
            let addr = cpu.fetch16(bus);
            cpu.write8(bus, addr, cpu.a);
            16
        }
        0xFA => {
            let addr = cpu.fetch16(bus);
            cpu.a = cpu.read8(bus, addr);
            16
        }

        // LDH (a8),A / LDH A,(a8)
        0xE0 => {
            let n = cpu.fetch8(bus) as u16;
            cpu.write8(bus, 0xFF00 | n, cpu.a);
            12
        }
        0xF0 => {
            let n = cpu.fetch8(bus) as u16;
            cpu.a = cpu.read8(bus, 0xFF00 | n);
            12
        }
        // LD (C),A / LD A,(C)
        0xE2 => {
            let addr = 0xFF00 | cpu.c as u16;
            cpu.write8(bus, addr, cpu.a);
            8
        }
        0xF2 => {
            let addr = 0xFF00 | cpu.c as u16;
            cpu.a = cpu.read8(bus, addr);
            8
        }

        // LD r,d8
        0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
            let r = r8_from_code((opcode >> 3) & 0x07);
            let v = cpu.fetch8(bus);
            cpu.write_r8(bus, r, v);
            if r == R8::HlInd {
                12
            } else {
                8
            }
        }

        // 0x40..=0x7F: LD r,r' (incl (HL) cases) and HALT (0x76)
        0x76 => {
            // HALT bug: when IME=0 and an interrupt is already pending, HALT does not
            // enter the halted state and the next opcode fetch does not increment PC.
            if !cpu.ime && pending_mask(bus.ie, bus.iflag) != 0 {
                cpu.halt_bug = true;
            } else {
                cpu.halted = true;
            }
            4
        }
        0x40..=0x7F => {
            let dst = r8_from_code((opcode >> 3) & 0x07);
            let src = r8_from_code(opcode & 0x07);
            let v = cpu.read_r8(bus, src);
            cpu.write_r8(bus, dst, v);
            if dst == R8::HlInd || src == R8::HlInd {
                8
            } else {
                4
            }
        }

        // INC/DEC r
        0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
            let r = r8_from_code((opcode >> 3) & 0x07);
            let v = cpu.read_r8(bus, r);
            let res = v.wrapping_add(1);
            cpu.write_r8(bus, r, res);
            cpu.set_flag(Flag::Z, res == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, (v & 0x0F) == 0x0F);
            if r == R8::HlInd {
                12
            } else {
                4
            }
        }
        0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
            let r = r8_from_code((opcode >> 3) & 0x07);
            let v = cpu.read_r8(bus, r);
            let res = v.wrapping_sub(1);
            cpu.write_r8(bus, r, res);
            cpu.set_flag(Flag::Z, res == 0);
            cpu.set_flag(Flag::N, true);
            cpu.set_flag(Flag::H, (v & 0x0F) == 0x00);
            if r == R8::HlInd {
                12
            } else {
                4
            }
        }

        // INC/DEC rr
        0x03 => {
            cpu.set_bc(cpu.bc().wrapping_add(1));
            8
        }
        0x13 => {
            cpu.set_de(cpu.de().wrapping_add(1));
            8
        }
        0x23 => {
            cpu.set_hl(cpu.hl().wrapping_add(1));
            8
        }
        0x33 => {
            cpu.sp = cpu.sp.wrapping_add(1);
            8
        }
        0x0B => {
            cpu.set_bc(cpu.bc().wrapping_sub(1));
            8
        }
        0x1B => {
            cpu.set_de(cpu.de().wrapping_sub(1));
            8
        }
        0x2B => {
            cpu.set_hl(cpu.hl().wrapping_sub(1));
            8
        }
        0x3B => {
            cpu.sp = cpu.sp.wrapping_sub(1);
            8
        }

        // ADD HL,rr
        0x09 | 0x19 | 0x29 | 0x39 => {
            let hl = cpu.hl();
            let rr = match opcode {
                0x09 => cpu.bc(),
                0x19 => cpu.de(),
                0x29 => cpu.hl(),
                _ => cpu.sp,
            };
            let sum = hl as u32 + rr as u32;
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ((hl & 0x0FFF) + (rr & 0x0FFF)) > 0x0FFF);
            cpu.set_flag(Flag::C, sum > 0xFFFF);
            cpu.set_hl(sum as u16);
            8
        }

        // 0x80..=0xBF: ALU A,r
        0x80..=0xBF => {
            let op = (opcode >> 3) & 0x07;
            let r = r8_from_code(opcode & 0x07);
            let v = cpu.read_r8(bus, r);
            let carry = if cpu.flag(Flag::C) { 1 } else { 0 };

            match op {
                0 => cpu.a = alu_add(cpu, cpu.a, v, 0),     // ADD
                1 => cpu.a = alu_add(cpu, cpu.a, v, carry), // ADC
                2 => cpu.a = alu_sub(cpu, cpu.a, v, 0),     // SUB
                3 => cpu.a = alu_sub(cpu, cpu.a, v, carry), // SBC
                4 => {
                    cpu.a &= v;
                    cpu.set_flag(Flag::Z, cpu.a == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, true);
                    cpu.set_flag(Flag::C, false);
                }
                5 => {
                    cpu.a ^= v;
                    cpu.set_flag(Flag::Z, cpu.a == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);
                    cpu.set_flag(Flag::C, false);
                }
                6 => {
                    cpu.a |= v;
                    cpu.set_flag(Flag::Z, cpu.a == 0);
                    cpu.set_flag(Flag::N, false);
                    cpu.set_flag(Flag::H, false);
                    cpu.set_flag(Flag::C, false);
                }
                _ => {
                    let _ = alu_sub(cpu, cpu.a, v, 0); // CP
                }
            }

            if r == R8::HlInd {
                8
            } else {
                4
            }
        }

        // Immediate ALU ops
        0xC6 => {
            let v = cpu.fetch8(bus);
            cpu.a = alu_add(cpu, cpu.a, v, 0);
            8
        }
        0xCE => {
            let v = cpu.fetch8(bus);
            let carry = if cpu.flag(Flag::C) { 1 } else { 0 };
            cpu.a = alu_add(cpu, cpu.a, v, carry);
            8
        }
        0xD6 => {
            let v = cpu.fetch8(bus);
            cpu.a = alu_sub(cpu, cpu.a, v, 0);
            8
        }
        0xDE => {
            let v = cpu.fetch8(bus);
            let carry = if cpu.flag(Flag::C) { 1 } else { 0 };
            cpu.a = alu_sub(cpu, cpu.a, v, carry);
            8
        }
        0xE6 => {
            let v = cpu.fetch8(bus);
            cpu.a &= v;
            cpu.set_flag(Flag::Z, cpu.a == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, true);
            cpu.set_flag(Flag::C, false);
            8
        }
        0xEE => {
            let v = cpu.fetch8(bus);
            cpu.a ^= v;
            cpu.set_flag(Flag::Z, cpu.a == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, false);
            8
        }
        0xF6 => {
            let v = cpu.fetch8(bus);
            cpu.a |= v;
            cpu.set_flag(Flag::Z, cpu.a == 0);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, false);
            8
        }
        0xFE => {
            let v = cpu.fetch8(bus);
            let _ = alu_sub(cpu, cpu.a, v, 0);
            8
        }

        // JR
        0x18 | 0x20 | 0x28 | 0x30 | 0x38 => {
            let off = cpu.fetch8(bus) as i8;
            if opcode == 0x18 || cond(cpu, opcode) {
                jr(cpu, off);
                12
            } else {
                8
            }
        }

        // JP
        0xC3 => {
            let addr = cpu.fetch16(bus);
            cpu.pc = addr;
            16
        }
        0xE9 => {
            cpu.pc = cpu.hl();
            4
        }
        0xC2 | 0xCA | 0xD2 | 0xDA => {
            let addr = cpu.fetch16(bus);
            if cond(cpu, opcode) {
                cpu.pc = addr;
                16
            } else {
                12
            }
        }

        // CALL
        0xCD => {
            let addr = cpu.fetch16(bus);
            cpu.push16(bus, cpu.pc);
            cpu.pc = addr;
            24
        }
        0xC4 | 0xCC | 0xD4 | 0xDC => {
            let addr = cpu.fetch16(bus);
            if cond(cpu, opcode) {
                cpu.push16(bus, cpu.pc);
                cpu.pc = addr;
                24
            } else {
                12
            }
        }

        // RET
        0xC9 => {
            cpu.pc = cpu.pop16(bus);
            16
        }
        0xC0 | 0xC8 | 0xD0 | 0xD8 => {
            if cond(cpu, opcode) {
                cpu.pc = cpu.pop16(bus);
                20
            } else {
                8
            }
        }
        0xD9 => {
            cpu.pc = cpu.pop16(bus);
            cpu.ime = true;
            16
        }

        // RST
        0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
            let vec = (opcode & 0x38) as u16;
            cpu.push16(bus, cpu.pc);
            cpu.pc = vec;
            16
        }

        // PUSH/POP
        0xC5 => {
            cpu.push16(bus, cpu.bc());
            16
        }
        0xD5 => {
            cpu.push16(bus, cpu.de());
            16
        }
        0xE5 => {
            cpu.push16(bus, cpu.hl());
            16
        }
        0xF5 => {
            cpu.push16(bus, cpu.af());
            16
        }
        0xC1 => {
            let v = cpu.pop16(bus);
            cpu.set_bc(v);
            12
        }
        0xD1 => {
            let v = cpu.pop16(bus);
            cpu.set_de(v);
            12
        }
        0xE1 => {
            let v = cpu.pop16(bus);
            cpu.set_hl(v);
            12
        }
        0xF1 => {
            let v = cpu.pop16(bus);
            cpu.set_af(v);
            12
        }

        // Misc
        0x27 => {
            daa(cpu);
            4
        }
        0x2F => {
            cpu.a = !cpu.a;
            cpu.set_flag(Flag::N, true);
            cpu.set_flag(Flag::H, true);
            4
        }
        0x37 => {
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, true);
            4
        }
        0x3F => {
            let c = cpu.flag(Flag::C);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, !c);
            4
        }
        0xF3 => {
            cpu.ime = false;
            cpu.ei_pending = false;
            4
        }
        0xFB => {
            cpu.ei_pending = true;
            4
        }

        // Rotate A
        0x07 => {
            let c = (cpu.a & 0x80) != 0;
            cpu.a = cpu.a.rotate_left(1);
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, c);
            4
        }
        0x0F => {
            let c = (cpu.a & 0x01) != 0;
            cpu.a = cpu.a.rotate_right(1);
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, c);
            4
        }
        0x17 => {
            let carry_in = if cpu.flag(Flag::C) { 1 } else { 0 };
            let c = (cpu.a & 0x80) != 0;
            cpu.a = cpu.a.wrapping_shl(1) | carry_in;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, c);
            4
        }
        0x1F => {
            let carry_in = if cpu.flag(Flag::C) { 0x80 } else { 0 };
            let c = (cpu.a & 0x01) != 0;
            cpu.a = (cpu.a >> 1) | carry_in;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, false);
            cpu.set_flag(Flag::C, c);
            4
        }

        // ADD SP,e8 / LD HL,SP+e8
        0xE8 | 0xF8 => {
            let e = cpu.fetch8(bus) as i8 as i16;
            let sp = cpu.sp;
            let res = (sp as i32 + e as i32) as u16;
            let e_u = e as u16;
            cpu.set_flag(Flag::Z, false);
            cpu.set_flag(Flag::N, false);
            cpu.set_flag(Flag::H, ((sp & 0x0F) + (e_u & 0x0F)) > 0x0F);
            cpu.set_flag(Flag::C, ((sp & 0xFF) + (e_u & 0xFF)) > 0xFF);
            if opcode == 0xE8 {
                cpu.sp = res;
            } else {
                cpu.set_hl(res);
            }
            if opcode == 0xE8 {
                16
            } else {
                12
            }
        }

        0xF9 => {
            cpu.sp = cpu.hl();
            8
        }

        // Undocumented/unused opcodes: treat as NOP.
        _ => 4,
    }
}
