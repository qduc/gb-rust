use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn temp_sav_path(prefix: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("gb-core-{prefix}-{}-{id}.sav", std::process::id()))
}

fn make_banked_rom(bank_count: usize) -> Vec<u8> {
    let mut rom = vec![0u8; bank_count * 0x4000];
    for bank in 0..bank_count {
        rom[bank * 0x4000] = bank as u8;
    }
    rom[0x0148] = match bank_count {
        1 => 0x00,
        2 => 0x01,
        4 => 0x02,
        8 => 0x03,
        16 => 0x04,
        32 => 0x05,
        64 => 0x06,
        128 => 0x07,
        512 => 0x08,
        _ => 0x00,
    };
    rom
}

fn remove_if_exists(path: &Path) {
    if path.exists() {
        let _ = std::fs::remove_file(path);
    }
}

#[test]
fn mbc3_battery_save_roundtrip_persists_ram_and_rtc() {
    let sav = temp_sav_path("mbc3");
    remove_if_exists(&sav);

    let mut rom = make_banked_rom(4);
    rom[0x0147] = 0x10; // MBC3 + Timer + RAM + Battery
    rom[0x0149] = 0x03; // 32KB RAM

    let cart = Cartridge::from_rom(rom.clone()).unwrap();
    let mut bus = Bus::new(cart);
    bus.write8(0x0000, 0x0A); // enable RAM/RTC

    // RAM content
    bus.write8(0x4000, 0x02);
    bus.write8(0xA123, 0x5A);

    // RTC content
    bus.write8(0x4000, 0x08);
    bus.write8(0xA000, 37);
    bus.write8(0x4000, 0x09);
    bus.write8(0xA000, 12);
    bus.write8(0x4000, 0x0A);
    bus.write8(0xA000, 7);
    bus.write8(0x4000, 0x0B);
    bus.write8(0xA000, 0xAA);
    bus.write8(0x4000, 0x0C);
    bus.write8(0xA000, 0x81);

    bus.save_to_path(&sav).unwrap();

    let cart2 = Cartridge::from_rom(rom).unwrap();
    let mut bus2 = Bus::new(cart2);
    bus2.load_from_path(&sav).unwrap();
    bus2.write8(0x0000, 0x0A);

    bus2.write8(0x4000, 0x02);
    assert_eq!(bus2.read8(0xA123), 0x5A);

    bus2.write8(0x4000, 0x08);
    assert_eq!(bus2.read8(0xA000), 37);
    bus2.write8(0x4000, 0x09);
    assert_eq!(bus2.read8(0xA000), 12);
    bus2.write8(0x4000, 0x0A);
    assert_eq!(bus2.read8(0xA000), 7);
    bus2.write8(0x4000, 0x0B);
    assert_eq!(bus2.read8(0xA000), 0xAA);
    bus2.write8(0x4000, 0x0C);
    assert_eq!(bus2.read8(0xA000) & 0xC1, 0x81);

    remove_if_exists(&sav);
}

#[test]
fn mbc2_battery_save_roundtrip_persists_internal_ram() {
    let sav = temp_sav_path("mbc2");
    remove_if_exists(&sav);

    let mut rom = make_banked_rom(16);
    rom[0x0147] = 0x06; // MBC2 + Battery
    rom[0x0149] = 0x00; // internal RAM only

    let cart = Cartridge::from_rom(rom.clone()).unwrap();
    let mut bus = Bus::new(cart);

    bus.write8(0x0000, 0x0A); // RAM enable (A8=0)
    bus.write8(0xA1FF, 0xAB); // only low nibble is stored
    assert_eq!(bus.read8(0xA1FF), 0xFB);
    bus.save_to_path(&sav).unwrap();

    let cart2 = Cartridge::from_rom(rom).unwrap();
    let mut bus2 = Bus::new(cart2);
    bus2.load_from_path(&sav).unwrap();
    bus2.write8(0x0000, 0x0A);
    assert_eq!(bus2.read8(0xA1FF), 0xFB);

    remove_if_exists(&sav);
}
