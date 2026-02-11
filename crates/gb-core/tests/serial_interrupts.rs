use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;
use gb_core::interrupt::Interrupt;

fn make_rom(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[..program.len()].copy_from_slice(program);
    rom
}

fn setup(program: &[u8]) -> (Cpu, Bus) {
    let cart = Cartridge::from_rom(make_rom(program)).unwrap();
    (Cpu::new(), Bus::new(cart))
}

#[test]
fn serial_transfer_requests_interrupt_after_delay() {
    let (_cpu, mut bus) = setup(&[0x00]);

    bus.write8(0xFF01, 0x55);
    bus.write8(0xFF02, 0x81); // start transfer, internal clock

    assert_eq!(bus.iflag & Interrupt::Serial.bit(), 0);

    bus.tick(4096);

    assert_ne!(bus.iflag & Interrupt::Serial.bit(), 0);
    assert_eq!(bus.read8(0xFF02) & 0x80, 0);

    let out = bus.serial.take_output();
    assert_eq!(out, vec![0x55]);
}

#[test]
fn halt_wakes_on_serial_pending_when_ime_false() {
    let (mut cpu, mut bus) = setup(&[0x00]); // NOP
    cpu.halted = true;
    cpu.ime = false;

    bus.ie = Interrupt::Serial.bit();
    bus.write8(0xFF01, 0x99);
    bus.write8(0xFF02, 0x81);

    bus.tick(4096);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert!(!cpu.halted);
    assert_eq!(cpu.pc, 1);
    assert_ne!(bus.iflag & Interrupt::Serial.bit(), 0);
}

#[test]
fn serial_interrupt_services_vector_when_ime_true() {
    let (mut cpu, mut bus) = setup(&[0x00]);
    cpu.halted = true;
    cpu.ime = true;
    cpu.pc = 0x1234;
    cpu.sp = 0xFFFE;

    bus.ie = Interrupt::Serial.bit();
    bus.write8(0xFF01, 0x42);
    bus.write8(0xFF02, 0x81);

    bus.tick(4096);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 20);
    assert_eq!(cpu.pc, 0x0058);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x34);
    assert_eq!(bus.read8(0xFFFD), 0x12);
    assert_eq!(bus.iflag & Interrupt::Serial.bit(), 0);
}
