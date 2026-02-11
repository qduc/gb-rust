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

    // Populate WRAM at 0xC000..0xC09F with a pattern that makes byte 0 observable.
    for i in 0..0xA0u16 {
        bus.write8(0xC000 + i, (i as u8).wrapping_add(1));
    }

    // Start DMA from 0xC000 page.
    bus.write8(0xFF46, 0xC0);

    // DMA should not complete instantly.
    assert_eq!(bus.oam[0], 0x00);

    // Hardware waits 1 M-cycle before the first byte transfer begins.
    bus.tick(4);
    assert_eq!(bus.oam[0], 0x00);

    // Then one byte is transferred every 4 cycles.
    bus.tick(4);
    assert_eq!(bus.oam[0], 0x01);
    assert_eq!(bus.oam[1], 0x00);

    // Finish remaining transfer window.
    bus.tick(4 * 0x9F);

    for i in 0..0xA0u16 {
        assert_eq!(bus.read8(0xFE00 + i), (i as u8).wrapping_add(1));
    }
}

#[test]
fn oam_dma_blocks_cpu_bus_except_hram() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xC000, 0x12);
    bus.write8(0xFF80, 0x34);
    bus.write8(0xFFFF, 0x1F);

    bus.write8(0xFF46, 0xC0);

    // Non-HRAM accesses are blocked while DMA is active.
    assert_eq!(bus.read8(0xC000), 0xFF);
    bus.write8(0xC000, 0x99);
    assert_eq!(bus.read8(0xC000), 0xFF);
    assert_eq!(bus.read8(0xFFFF), 0xFF);
    bus.write8(0xFFFF, 0x00);
    assert_eq!(bus.read8(0xFFFF), 0xFF);

    // HRAM remains accessible.
    assert_eq!(bus.read8(0xFF80), 0x34);
    bus.write8(0xFF80, 0x56);
    assert_eq!(bus.read8(0xFF80), 0x56);

    // Once DMA completes (160 bytes + 1 M-cycle startup delay), normal access resumes.
    bus.tick(4 * 0xA1);
    assert_eq!(bus.read8(0xC000), 0x12);
    assert_eq!(bus.read8(0xFFFF), 0x1F);
    bus.write8(0xC000, 0x99);
    assert_eq!(bus.read8(0xC000), 0x99);
}

#[test]
fn oam_dma_start_timing_can_miss_current_scanline_sprite_fetch() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    // Tile 0 row 0 -> color 3 at leftmost pixel.
    bus.vram[0] = 0x80;
    bus.vram[1] = 0x80;

    // DMA source sprite entry at C000: y=16, x=8, tile=0, attrs=0.
    bus.write8(0xC000, 16);
    bus.write8(0xC001, 8);
    bus.write8(0xC002, 0);
    bus.write8(0xC003, 0);

    // LCD on + OBJ on (BG off), default palettes.
    bus.write8(0xFF40, 0x82);
    bus.write8(0xFF48, 0xE4);

    // Enter line 0 mode 2 and advance close to mode 3 boundary (dot 80).
    bus.tick(0);
    bus.tick(76);
    assert_eq!(bus.read8(0xFF41) & 0x03, 2);

    // Start DMA too late: after 4 cycles the startup delay hasn't elapsed, so no bytes are copied.
    bus.write8(0xFF46, 0xC0);
    bus.tick(4);

    // Sprite X has not yet copied by mode 3 start, so pixel remains white.
    assert_eq!(bus.ppu.framebuffer()[0], 0xFFFF_FFFF);
}
