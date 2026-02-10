use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

fn make_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0148] = 0x00; // 32KB
    rom
}

#[test]
fn stat_write_masks_lower_bits() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    // Avoid coincidence bit being set by default.
    bus.write8(0xFF45, 1);
    bus.tick(0);

    let before = bus.read8(0xFF41);

    bus.write8(0xFF41, 0xFF);
    let after = bus.read8(0xFF41);

    assert_eq!(after & 0x07, before & 0x07);
    assert_eq!(after & 0x78, 0x78);
}

#[test]
fn ly_write_resets_to_zero() {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFF40, 0x80); // LCD on
    bus.tick(1);

    bus.tick(456 * 10);
    assert_eq!(bus.read8(0xFF44), 10);

    bus.write8(0xFF44, 0x00);
    assert_eq!(bus.read8(0xFF44), 0);

    bus.tick(456);
    assert_eq!(bus.read8(0xFF44), 1);
}
