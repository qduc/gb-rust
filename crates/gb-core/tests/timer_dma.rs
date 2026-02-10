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
fn div_increments_and_resets_on_write() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF04), 0x00);

    bus.tick(256);
    assert_eq!(bus.read8(0xFF04), 0x01);

    bus.write8(0xFF04, 0x00);
    assert_eq!(bus.read8(0xFF04), 0x00);
}

#[test]
fn div_write_triggers_tima_on_falling_edge() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    // Enable timer at 16-cycle rate (input bit 3).
    bus.write8(0xFF07, 0x05);
    bus.write8(0xFF05, 0x00);

    // Counter=8 => selected input bit is high.
    bus.tick(8);
    bus.write8(0xFF04, 0x00);

    // DIV reset creates old=1 -> new=0 transition, so TIMA increments.
    assert_eq!(bus.read8(0xFF05), 0x01);
}

#[test]
fn tima_increments_at_selected_frequency() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    // Enable timer, fastest frequency (262144 Hz => 16 cycles per increment).
    bus.write8(0xFF07, 0x05);

    bus.tick(16);
    assert_eq!(bus.read8(0xFF05), 0x01);

    bus.tick(16);
    assert_eq!(bus.read8(0xFF05), 0x02);
}

#[test]
fn tac_write_triggers_tima_on_falling_edge() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFF05, 0x00);
    bus.write8(0xFF07, 0x05); // enabled, input bit 3
    bus.tick(8); // selected input bit is high

    // Disable timer; old=1 -> new=0 should increment TIMA once.
    bus.write8(0xFF07, 0x00);
    assert_eq!(bus.read8(0xFF05), 0x01);
}

#[test]
fn tima_overflow_reloads_tma_and_requests_interrupt() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFF06, 0xAB); // TMA
    bus.write8(0xFF05, 0xFF); // TIMA
    bus.write8(0xFF07, 0x05); // enable + fastest

    bus.tick(16);

    assert_eq!(bus.read8(0xFF05), 0xAB);
    assert_ne!(bus.iflag & (1 << 2), 0);
}

#[test]
fn timer_interrupt_can_be_serviced_by_cpu() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);
    let mut cpu = Cpu::new();

    cpu.pc = 0x1234;
    cpu.sp = 0xFFFE;
    cpu.ime = true;

    bus.ie = 1 << 2; // Timer

    bus.write8(0xFF06, 0x77); // TMA
    bus.write8(0xFF05, 0xFF); // TIMA
    bus.write8(0xFF07, 0x05); // enable + fastest

    // Trigger overflow => request interrupt.
    bus.tick(16);
    assert_ne!(bus.iflag & (1 << 2), 0);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 20);
    assert_eq!(cpu.pc, 0x0050);
    assert_eq!(bus.iflag & (1 << 2), 0);

    // Ensure PC was pushed.
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x34);
    assert_eq!(bus.read8(0xFFFD), 0x12);
}

#[test]
fn oam_dma_copies_0xa0_bytes() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    // Populate WRAM at 0xC000..0xC09F.
    for i in 0..0xA0u16 {
        bus.write8(0xC000 + i, (i & 0xFF) as u8);
    }

    // Start DMA from 0xC000 page.
    bus.write8(0xFF46, 0xC0);

    for i in 0..0xA0u16 {
        assert_eq!(bus.read8(0xFE00 + i), (i & 0xFF) as u8);
    }
}
