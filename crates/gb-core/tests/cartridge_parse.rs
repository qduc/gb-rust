use gb_core::cartridge::header::HeaderError;
use gb_core::cartridge::{Cartridge, CartridgeError};

#[test]
fn rejects_rom_smaller_than_header_region() {
    let rom = vec![0u8; 0x0100];
    match Cartridge::from_rom(rom) {
        Err(CartridgeError::HeaderParse(HeaderError::RomTooSmall)) => {}
        Err(_) => panic!("unexpected error"),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn rejects_rom_shorter_than_declared_size() {
    let mut rom = vec![0u8; 0x4000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x01; // Declares 64KB
    rom[0x0149] = 0x00; // No RAM

    match Cartridge::from_rom(rom) {
        Err(CartridgeError::RomTooSmall {
            declared: 0x8000,
            actual: 0x4000,
        }) => {}
        Err(_) => panic!("unexpected error"),
        Ok(_) => panic!("expected size error"),
    }
}

#[test]
fn rejects_unsupported_cartridge_type() {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0xFF; // Unsupported type
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;

    match Cartridge::from_rom(rom) {
        Err(CartridgeError::HeaderParse(HeaderError::UnsupportedCartridgeType(0xFF))) => {}
        Err(_) => panic!("unexpected error"),
        Ok(_) => panic!("expected parse error"),
    }
}

#[test]
fn accepts_mbc2_and_mbc5_cartridge_types() {
    let mut mbc2_rom = vec![0u8; 0x4000];
    mbc2_rom[0x0147] = 0x06; // MBC2 + battery
    mbc2_rom[0x0148] = 0x00;
    mbc2_rom[0x0149] = 0x00;
    assert!(Cartridge::from_rom(mbc2_rom).is_ok());

    let mut mbc5_rom = vec![0u8; 0x4000 * 512];
    mbc5_rom[0x0147] = 0x1B; // MBC5 + RAM + battery
    mbc5_rom[0x0148] = 0x08; // 8MB
    mbc5_rom[0x0149] = 0x03; // 32KB RAM
    assert!(Cartridge::from_rom(mbc5_rom).is_ok());
}
