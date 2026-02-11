use gb_core::bus::Bus;
use gb_core::cartridge::header::CgbSupport;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;

fn make_rom(cgb_flag: u8, program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[0x0143] = cgb_flag;
    rom[..program.len()].copy_from_slice(program);
    rom
}

#[test]
fn cartridge_parse_reports_cgb_capability_from_header_0143() {
    let dmg = Cartridge::from_rom(make_rom(0x00, &[])).unwrap();
    assert_eq!(dmg.header.cgb_support, CgbSupport::DmgOnly);

    let compat = Cartridge::from_rom(make_rom(0x80, &[])).unwrap();
    assert_eq!(compat.header.cgb_support, CgbSupport::CgbCompatible);

    let cgb_only = Cartridge::from_rom(make_rom(0xC0, &[])).unwrap();
    assert_eq!(cgb_only.header.cgb_support, CgbSupport::CgbOnly);

    let cgb_only_extra_bits = Cartridge::from_rom(make_rom(0xC1, &[])).unwrap();
    assert_eq!(cgb_only_extra_bits.header.cgb_support, CgbSupport::CgbOnly);
}

#[test]
fn key1_reports_speed_and_prepare_bits_in_cgb_mode() {
    let cart = Cartridge::from_rom(make_rom(0x80, &[])).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF4D), 0x7E);

    bus.write8(0xFF4D, 0x01);
    assert_eq!(bus.read8(0xFF4D), 0x7F);

    bus.write8(0xFF4D, 0x00);
    assert_eq!(bus.read8(0xFF4D), 0x7E);
}

#[test]
fn key1_ignores_writes_in_dmg_mode() {
    let cart = Cartridge::from_rom(make_rom(0x00, &[])).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF4D), 0xFF);
    bus.write8(0xFF4D, 0x01);
    assert_eq!(bus.read8(0xFF4D), 0xFF);
}

#[test]
fn stop_switches_cpu_speed_only_when_key1_prepare_is_set() {
    // STOP 00 ; NOP
    let cart = Cartridge::from_rom(make_rom(0x80, &[0x10, 0x00, 0x00])).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    bus.write8(0xFF4D, 0x01);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 8);
    assert_eq!(cpu.pc, 2);
    assert!(!cpu.halted);

    // Bit7 (current speed) should now be set, and prepare bit cleared.
    assert_eq!(bus.read8(0xFF4D), 0xFE);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 3);
}

#[test]
fn stop_keeps_existing_behavior_without_cgb_speed_switch_request() {
    let cart = Cartridge::from_rom(make_rom(0x80, &[0x10, 0x00, 0x00])).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 8);
    assert_eq!(cpu.pc, 2);
    assert!(cpu.halted);
    assert_eq!(bus.read8(0xFF4D), 0x7E);
}

#[test]
fn dmg_rom_does_not_expose_cgb_speed_switch_side_effects() {
    let cart = Cartridge::from_rom(make_rom(0x00, &[0x10, 0x00])).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    // Writing KEY1 on DMG should have no effect.
    bus.write8(0xFF4D, 0x01);

    cpu.step(&mut bus);
    assert!(cpu.halted);
    assert_eq!(bus.read8(0xFF4D), 0xFF);
}
