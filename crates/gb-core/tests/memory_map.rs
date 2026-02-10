use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

// Helper to create a banked ROM with each bank marked
fn make_banked_rom(bank_count: usize) -> Vec<u8> {
    let mut rom = vec![0u8; bank_count * 0x4000];
    for bank in 0..bank_count {
        rom[bank * 0x4000] = bank as u8;
    }
    // Set minimal header
    rom[0x0148] = match bank_count {
        1 => 0x00,
        2 => 0x01,
        4 => 0x02,
        8 => 0x03,
        16 => 0x04,
        32 => 0x05,
        64 => 0x06,
        128 => 0x07,
        _ => 0x00,
    };
    rom
}

#[test]
fn mbc0_rom_reads_map_directly() {
    let rom = make_banked_rom(2);

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0x0000), 0x00, "Bank 0 marker");
    assert_eq!(bus.read8(0x4000), 0x01, "Bank 1 marker");
}

#[test]
fn mbc0_ram_reads_write_through_when_present() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // MBC0 with no RAM should read 0xFF
    assert_eq!(bus.read8(0xA000), 0xFF);
    bus.write8(0xA000, 0x42);
    assert_eq!(bus.read8(0xA000), 0xFF);
}

#[test]
fn mbc0_external_ram_write_read() {
    // Create a ROM with RAM
    let mut rom = vec![0x00; 0x4000];
    rom[0x0147] = 0x00;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x02; // 8KB RAM

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xA000, 0x55);
    assert_eq!(bus.read8(0xA000), 0x55);

    bus.write8(0xA001, 0xAA);
    assert_eq!(bus.read8(0xA001), 0xAA);
}

#[test]
fn wram_and_echo_are_mirrored() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Write to WRAM, read from echo
    bus.write8(0xC000, 0x42);
    assert_eq!(bus.read8(0xE000), 0x42);

    // Write to echo, read from WRAM
    bus.write8(0xE123, 0x99);
    assert_eq!(bus.read8(0xC123), 0x99);
}

#[test]
fn hram_ie_if_registers_map() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Test HRAM
    bus.write8(0xFF80, 0x11);
    assert_eq!(bus.read8(0xFF80), 0x11);

    bus.write8(0xFFFE, 0x22);
    assert_eq!(bus.read8(0xFFFE), 0x22);

    // Test IF register (0xFF0F)
    bus.write8(0xFF0F, 0x0F);
    assert_eq!(bus.read8(0xFF0F), 0xEF);

    // Test IE register (0xFFFF)
    bus.write8(0xFFFF, 0xE0);
    assert_eq!(bus.read8(0xFFFF), 0xE0);
}

#[test]
fn mbc1_defaults_to_bank1_in_0x4000_region() {
    let mut rom = make_banked_rom(4);
    rom[0x0147] = 0x01; // MBC1
    rom[0x0148] = 0x02; // 128KB ROM = 4 banks
    rom[0x0149] = 0x00; // No RAM

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Default should be bank 1 visible at 0x4000
    assert_eq!(bus.read8(0x4000), 0x01);
}

#[test]
fn mbc1_rom_bank_switch_low5() {
    let mut rom = make_banked_rom(8);
    rom[0x0147] = 0x01; // MBC1
    rom[0x0148] = 0x03; // 256KB ROM = 8 banks
    rom[0x0149] = 0x00; // No RAM

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Switch to bank 2
    bus.write8(0x2000, 0x02);
    assert_eq!(bus.read8(0x4000), 0x02);

    // Try to switch to bank 0 (should wrap to bank 1)
    bus.write8(0x2000, 0x00);
    assert_eq!(bus.read8(0x4000), 0x01);

    // Switch to bank 3
    bus.write8(0x2000, 0x03);
    assert_eq!(bus.read8(0x4000), 0x03);
}

#[test]
fn mbc1_rom_bank_switch_uses_high_bits() {
    let mut rom = make_banked_rom(128);
    rom[0x0147] = 0x01; // MBC1
    rom[0x0148] = 0x07; // 4MB ROM = 128 banks
    rom[0x0149] = 0x00; // No RAM

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Set high bits via 0x4000: bank_high2 = 0x01 (bits 5-6)
    bus.write8(0x4000, 0x01);
    // Set low bits via 0x2000: rom_bank_low5 = 0x01
    bus.write8(0x2000, 0x01);
    // Bank = (0x01 << 5) | 0x01 = 0x21 = 33
    assert_eq!(bus.read8(0x4000), 33_u8);
}

#[test]
fn mbc1_ram_enable_disable() {
    let mut rom = vec![0x00; 0x4000];
    rom[0x0147] = 0x02; // MBC1 with RAM
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x02; // 8KB RAM

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // RAM disabled by default, should read 0xFF
    assert_eq!(bus.read8(0xA000), 0xFF);

    // Enable RAM
    bus.write8(0x0000, 0x0A);
    bus.write8(0xA000, 0x42);
    assert_eq!(bus.read8(0xA000), 0x42);

    // Disable RAM
    bus.write8(0x0000, 0x00);
    assert_eq!(bus.read8(0xA000), 0xFF);
}

#[test]
fn mbc1_ram_bank_switch_mode1() {
    let mut rom = vec![0x00; 0x4000];
    rom[0x0147] = 0x03; // MBC1 with RAM + Battery
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x03; // 32KB RAM (4 banks)

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Enable RAM
    bus.write8(0x0000, 0x0A);

    // Set mode 1 (RAM banking)
    bus.write8(0x6000, 0x01);

    // Write to RAM bank 0
    bus.write8(0xA000, 0x11);

    // Switch to RAM bank 1
    bus.write8(0x4000, 0x01);
    bus.write8(0xA000, 0x22);

    // Switch back to RAM bank 0
    bus.write8(0x4000, 0x00);
    assert_eq!(bus.read8(0xA000), 0x11);

    // Switch to RAM bank 1 again
    bus.write8(0x4000, 0x01);
    assert_eq!(bus.read8(0xA000), 0x22);
}

#[test]
fn mbc3_ram_banking_and_rtc_select() {
    let mut rom = make_banked_rom(2);
    rom[0x0147] = 0x12; // MBC3 + RAM
    rom[0x0149] = 0x03; // 32KB RAM (4 banks)

    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    // Enable RAM/RTC
    bus.write8(0x0000, 0x0A);

    // RAM bank 0
    bus.write8(0x4000, 0x00);
    bus.write8(0xA000, 0x11);

    // RAM bank 1
    bus.write8(0x4000, 0x01);
    bus.write8(0xA000, 0x22);

    // Verify banked reads
    bus.write8(0x4000, 0x00);
    assert_eq!(bus.read8(0xA000), 0x11);
    bus.write8(0x4000, 0x01);
    assert_eq!(bus.read8(0xA000), 0x22);

    // RTC select should read as 0 and not clobber RAM
    bus.write8(0x4000, 0x08);
    assert_eq!(bus.read8(0xA000), 0x00);
    bus.write8(0xA000, 0x99);

    bus.write8(0x4000, 0x00);
    assert_eq!(bus.read8(0xA000), 0x11);
}

#[test]
fn vram_read_write() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0x8000, 0xAB);
    assert_eq!(bus.read8(0x8000), 0xAB);

    bus.write8(0x9FFF, 0xCD);
    assert_eq!(bus.read8(0x9FFF), 0xCD);
}

#[test]
fn oam_read_write() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0xFE00, 0x12);
    assert_eq!(bus.read8(0xFE00), 0x12);

    bus.write8(0xFE9F, 0x34);
    assert_eq!(bus.read8(0xFE9F), 0x34);
}

#[test]
fn unusable_region_reads_ff_ignores_writes() {
    let rom = vec![0x00; 0x4000];
    let cart = Cartridge::from_rom(rom).unwrap();
    let mut bus = Bus::new(cart);

    assert_eq!(bus.read8(0xFEA0), 0xFF);
    assert_eq!(bus.read8(0xFEFF), 0xFF);

    bus.write8(0xFEA0, 0x55);
    assert_eq!(bus.read8(0xFEA0), 0xFF);
}
