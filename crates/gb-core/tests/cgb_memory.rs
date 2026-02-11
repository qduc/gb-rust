use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

fn make_rom(cgb_flag: u8) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[0x0143] = cgb_flag;
    rom
}

#[test]
fn vbk_selects_cpu_visible_vram_bank_in_cgb_mode() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF4F), 0xFE);

    bus.write8(0x8000, 0x11);

    bus.write8(0xFF4F, 0x01);
    assert_eq!(bus.read8(0xFF4F), 0xFF);
    bus.write8(0x8000, 0x22);

    bus.write8(0xFF4F, 0x00);
    assert_eq!(bus.read8(0x8000), 0x11);

    bus.write8(0xFF4F, 0x01);
    assert_eq!(bus.read8(0x8000), 0x22);
}

#[test]
fn vbk_is_gated_in_dmg_mode() {
    let cart = Cartridge::from_rom(make_rom(0x00)).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF4F), 0xFF);

    bus.write8(0x8000, 0x33);
    bus.write8(0xFF4F, 0x01);
    assert_eq!(bus.read8(0xFF4F), 0xFF);
    assert_eq!(bus.read8(0x8000), 0x33);
}

#[test]
fn svbk_selects_switchable_wram_bank_in_cgb_mode() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    // Default bank is 1.
    assert_eq!(bus.read8(0xFF70), 0xF9);
    bus.write8(0xD000, 0x11);

    bus.write8(0xFF70, 0x02);
    assert_eq!(bus.read8(0xFF70), 0xFA);
    bus.write8(0xD000, 0x22);

    // Bank 0 request maps to bank 1.
    bus.write8(0xFF70, 0x00);
    assert_eq!(bus.read8(0xFF70), 0xF9);
    assert_eq!(bus.read8(0xD000), 0x11);

    bus.write8(0xFF70, 0x02);
    assert_eq!(bus.read8(0xD000), 0x22);
}

#[test]
fn c000_bank_is_fixed_regardless_of_svbk() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xC000, 0xA5);

    bus.write8(0xFF70, 0x03);
    assert_eq!(bus.read8(0xC000), 0xA5);

    bus.write8(0xFF70, 0x07);
    assert_eq!(bus.read8(0xC000), 0xA5);
}

#[test]
fn svbk_is_gated_in_dmg_mode() {
    let cart = Cartridge::from_rom(make_rom(0x00)).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFF70), 0xFF);

    bus.write8(0xD000, 0x44);
    bus.write8(0xFF70, 0x06);
    assert_eq!(bus.read8(0xFF70), 0xFF);
    assert_eq!(bus.read8(0xD000), 0x44);
}
