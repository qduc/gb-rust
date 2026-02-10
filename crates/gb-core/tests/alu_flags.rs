use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::cpu::Flag;
use gb_core::cpu::Cpu;

fn make_rom(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x4000];
    // Minimal header for Cartridge::from_rom
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[..program.len()].copy_from_slice(program);
    rom
}

fn setup(program: &[u8]) -> (Cpu, Bus) {
    let cart = Cartridge::from_rom(make_rom(program)).unwrap();
    let bus = Bus::new(cart);
    let cpu = Cpu::new();
    (cpu, bus)
}

fn assert_flags(cpu: &Cpu, z: bool, n: bool, h: bool, c: bool) {
    assert_eq!(cpu.flag(Flag::Z), z, "Z");
    assert_eq!(cpu.flag(Flag::N), n, "N");
    assert_eq!(cpu.flag(Flag::H), h, "H");
    assert_eq!(cpu.flag(Flag::C), c, "C");
}

#[test]
fn add_a_n_sets_znch() {
    // Half-carry, no carry.
    let (mut cpu, mut bus) = setup(&[0xC6, 0x01]); // ADD A,0x01
    cpu.a = 0x0F;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x10);
    assert_flags(&cpu, false, false, true, false);

    // Half-carry + carry + zero.
    let (mut cpu, mut bus) = setup(&[0xC6, 0x01]);
    cpu.a = 0xFF;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, false, true, true);
}

#[test]
fn adc_a_n_uses_carry_in() {
    // 0x0F + 0x00 + carry => 0x10 (half-carry set)
    let (mut cpu, mut bus) = setup(&[0xCE, 0x00]); // ADC A,0x00
    cpu.a = 0x0F;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x10);
    assert_flags(&cpu, false, false, true, false);

    // 0xFF + 0x00 + carry => 0x00 (carry + zero)
    let (mut cpu, mut bus) = setup(&[0xCE, 0x00]);
    cpu.a = 0xFF;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, false, true, true);
}

#[test]
fn sub_a_n_sets_znch() {
    // Half-borrow, no borrow.
    let (mut cpu, mut bus) = setup(&[0xD6, 0x01]); // SUB A,0x01
    cpu.a = 0x10;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x0F);
    assert_flags(&cpu, false, true, true, false);

    // Half-borrow + borrow.
    let (mut cpu, mut bus) = setup(&[0xD6, 0x01]);
    cpu.a = 0x00;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
    assert_flags(&cpu, false, true, true, true);
}

#[test]
fn sbc_a_n_uses_carry_in() {
    // 0x10 - 0x0F - carry => 0x00
    let (mut cpu, mut bus) = setup(&[0xDE, 0x0F]); // SBC A,0x0F
    cpu.a = 0x10;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, true, true, false);

    // 0x00 - 0x00 - carry => 0xFF (borrow)
    let (mut cpu, mut bus) = setup(&[0xDE, 0x00]);
    cpu.a = 0x00;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xFF);
    assert_flags(&cpu, false, true, true, true);
}

#[test]
fn and_xor_or_flags() {
    let (mut cpu, mut bus) = setup(&[0xE6, 0x0F]); // AND 0x0F
    cpu.a = 0xF0;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, false, true, false);

    let (mut cpu, mut bus) = setup(&[0xEE, 0xFF]); // XOR 0xFF
    cpu.a = 0xFF;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, false, false, false);

    let (mut cpu, mut bus) = setup(&[0xF6, 0x00]); // OR 0x00
    cpu.a = 0x00;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, true, false, false, false);
}

#[test]
fn cp_a_n_sets_flags_without_changing_a() {
    let (mut cpu, mut bus) = setup(&[0xFE, 0x3C]); // CP 0x3C
    cpu.a = 0x3C;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x3C);
    assert_flags(&cpu, true, true, false, false);

    let (mut cpu, mut bus) = setup(&[0xFE, 0x01]); // CP 0x01
    cpu.a = 0x00;
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x00);
    assert_flags(&cpu, false, true, true, true);
}

#[test]
fn inc_dec_r8_sets_znh_and_preserves_c() {
    // INC B: half-carry and preserve carry.
    let (mut cpu, mut bus) = setup(&[0x04]); // INC B
    cpu.b = 0x0F;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.b, 0x10);
    assert_flags(&cpu, false, false, true, true);

    // DEC B: half-borrow and preserve carry.
    let (mut cpu, mut bus) = setup(&[0x05]); // DEC B
    cpu.b = 0x10;
    cpu.set_flag(Flag::C, true);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.b, 0x0F);
    assert_flags(&cpu, false, true, true, true);

    // INC B: zero result.
    let (mut cpu, mut bus) = setup(&[0x04]);
    cpu.b = 0xFF;
    cpu.set_flag(Flag::C, false);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.b, 0x00);
    assert_flags(&cpu, true, false, true, false);

    // DEC B: zero result without half-borrow.
    let (mut cpu, mut bus) = setup(&[0x05]);
    cpu.b = 0x01;
    cpu.set_flag(Flag::C, false);
    cpu.pc = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.b, 0x00);
    assert_flags(&cpu, true, true, false, false);
}
