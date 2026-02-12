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

fn enter_hblank(bus: &mut Bus) {
    // Enable LCD and initialize timing state.
    bus.write8(0xFF40, 0x80);
    bus.tick(0);

    // Freshly enabled LCD starts at dot=0 in mode 2.
    bus.tick(80);
    bus.tick(172);
}

fn advance_to_next_hblank(bus: &mut Bus) {
    // Finish current line (remaining mode 0 cycles).
    bus.tick(204);
    // Next line mode 2 -> mode 3 -> mode 0.
    bus.tick(80);
    bus.tick(172);
}

#[test]
fn hdma_registers_are_gated_in_dmg_mode() {
    let cart = Cartridge::from_rom(make_rom(0x00)).unwrap();
    let mut bus = Bus::new(cart);

    for addr in 0xFF51..=0xFF55 {
        assert_eq!(bus.read8(addr), 0xFF);
        bus.write8(addr, 0x12);
        assert_eq!(bus.read8(addr), 0xFF);
    }
}

#[test]
fn gdma_copies_requested_blocks_and_completes() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    for i in 0..0x20u16 {
        bus.write8(0xC120 + i, (i as u8).wrapping_mul(3));
    }

    bus.write8(0xFF51, 0xC1);
    bus.write8(0xFF52, 0x2F); // low nibble ignored -> 0x20
    bus.write8(0xFF53, 0x01);
    bus.write8(0xFF54, 0x2D); // low nibble ignored -> 0x20
    bus.write8(0xFF55, 0x01); // GDMA, 2 blocks (0x20 bytes)

    for i in 0..0x20u16 {
        assert_eq!(bus.read8(0x8120 + i), (i as u8).wrapping_mul(3));
    }

    assert_eq!(bus.read8(0xFF55), 0xFF);
}

#[test]
fn hdma_transfers_one_block_per_hblank() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    for i in 0..0x20u16 {
        bus.write8(0xC200 + i, 0x80u8.wrapping_add(i as u8));
    }

    bus.write8(0xFF51, 0xC2);
    bus.write8(0xFF52, 0x00);
    bus.write8(0xFF53, 0x02);
    bus.write8(0xFF54, 0x00);
    bus.write8(0xFF55, 0x81); // HDMA, 2 blocks

    // No transfer before entering HBlank.
    assert_eq!(bus.read8(0x8200), 0x00);

    enter_hblank(&mut bus);
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8200 + i), 0x80u8.wrapping_add(i as u8));
    }
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8210 + i), 0x00);
    }
    assert_eq!(bus.read8(0xFF55), 0x00);

    // Extra cycles in the same HBlank should not trigger another block.
    bus.tick(8);
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8210 + i), 0x00);
    }

    advance_to_next_hblank(&mut bus);
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8210 + i), 0x90u8.wrapping_add(i as u8));
    }
    assert_eq!(bus.read8(0xFF55), 0xFF);
}

#[test]
fn hdma_can_be_terminated_by_writing_bit7_clear() {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    let mut bus = Bus::new(cart);

    for i in 0..0x20u16 {
        bus.write8(0xC300 + i, 0x40u8.wrapping_add(i as u8));
    }

    bus.write8(0xFF51, 0xC3);
    bus.write8(0xFF52, 0x00);
    bus.write8(0xFF53, 0x03);
    bus.write8(0xFF54, 0x00);
    bus.write8(0xFF55, 0x81); // HDMA, 2 blocks

    enter_hblank(&mut bus);
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8300 + i), 0x40u8.wrapping_add(i as u8));
    }

    bus.write8(0xFF55, 0x00); // terminate HDMA
    assert_eq!(bus.read8(0xFF55), 0x80);

    advance_to_next_hblank(&mut bus);
    for i in 0..0x10u16 {
        assert_eq!(bus.read8(0x8310 + i), 0x00);
    }
}
