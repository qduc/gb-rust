use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

fn make_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0148] = 0x00; // 32KB
    rom
}

#[test]
fn vram_is_blocked_for_cpu_during_mode3_and_restored_in_mode0() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0x8000, 0x12);

    bus.write8(0xFF40, 0x80); // LCD on
    bus.tick(0);
    assert_eq!(bus.read8(0xFF41) & 0x03, 2);

    bus.tick(80);
    assert_eq!(bus.read8(0xFF41) & 0x03, 3);

    assert_eq!(bus.read8(0x8000), 0xFF);
    bus.write8(0x8000, 0x34);
    assert_eq!(bus.read8(0x8000), 0xFF);

    bus.tick(172);
    assert_eq!(bus.read8(0xFF41) & 0x03, 0);
    assert_eq!(bus.read8(0x8000), 0x12);
}

#[test]
fn oam_is_blocked_for_cpu_during_mode2_and_mode3() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFE00, 0x56);

    bus.write8(0xFF40, 0x80); // LCD on
    bus.tick(0);
    assert_eq!(bus.read8(0xFF41) & 0x03, 2);

    assert_eq!(bus.read8(0xFE00), 0xFF);
    bus.write8(0xFE00, 0x99);

    bus.tick(80);
    assert_eq!(bus.read8(0xFF41) & 0x03, 3);
    assert_eq!(bus.read8(0xFE00), 0xFF);

    bus.tick(172);
    assert_eq!(bus.read8(0xFF41) & 0x03, 0);
    assert_eq!(bus.read8(0xFE00), 0x56);
}

#[test]
fn vram_and_oam_are_accessible_when_lcd_is_disabled() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFF40, 0x00); // LCD off
    bus.tick(0);

    bus.write8(0x8000, 0xAA);
    bus.write8(0xFE00, 0xBB);

    assert_eq!(bus.read8(0x8000), 0xAA);
    assert_eq!(bus.read8(0xFE00), 0xBB);
}
