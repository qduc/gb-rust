use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;

fn make_rom() -> Vec<u8> {
    // Minimal 32KB ROM with header bytes set enough for parsing.
    let mut rom = vec![0u8; 0x8000];
    rom[0x0148] = 0x00; // 32KB
    rom
}

#[test]
fn services_interrupt_pushes_pc_and_jumps() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    cpu.pc = 0x1234;
    cpu.sp = 0xFFFE;
    cpu.ime = true;

    bus.ie = 0x01;
    bus.iflag = 0x01;

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 20);
    assert_eq!(cpu.pc, 0x0040);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x34);
    assert_eq!(bus.read8(0xFFFD), 0x12);
    assert_eq!(bus.iflag & 0x01, 0);
    assert!(!cpu.ime);
}

#[test]
fn services_highest_priority_interrupt() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    cpu.pc = 0x2000;
    cpu.sp = 0xFFFE;
    cpu.ime = true;

    bus.ie = 0x1F;
    bus.iflag = (1 << 2) | (1 << 0); // Timer + VBlank

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 20);
    assert_eq!(cpu.pc, 0x0040);
    assert_eq!(bus.iflag & (1 << 0), 0);
    assert_ne!(bus.iflag & (1 << 2), 0);
}
