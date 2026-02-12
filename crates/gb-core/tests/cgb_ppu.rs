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

fn setup_cgb_bus() -> Bus {
    let cart = Cartridge::from_rom(make_rom(0x80)).unwrap();
    Bus::new(cart)
}

fn setup_dmg_bus() -> Bus {
    let cart = Cartridge::from_rom(make_rom(0x00)).unwrap();
    Bus::new(cart)
}

fn write_bg_palette_color(bus: &mut Bus, palette: u8, color: u8, bgr15: u16) {
    let index = (palette as usize) * 8 + (color as usize) * 2;
    bus.write8(0xFF68, index as u8);
    let [lo, hi] = bgr15.to_le_bytes();
    bus.write8(0xFF69, lo);
    bus.write8(0xFF68, (index as u8).wrapping_add(1));
    bus.write8(0xFF69, hi);
}

fn write_obj_palette_color(bus: &mut Bus, palette: u8, color: u8, bgr15: u16) {
    let index = (palette as usize) * 8 + (color as usize) * 2;
    bus.write8(0xFF6A, index as u8);
    let [lo, hi] = bgr15.to_le_bytes();
    bus.write8(0xFF6B, lo);
    bus.write8(0xFF6A, (index as u8).wrapping_add(1));
    bus.write8(0xFF6B, hi);
}

#[test]
fn cgb_bg_palette_registers_support_index_and_auto_increment() {
    let mut bus = setup_cgb_bus();

    assert_eq!(bus.read8(0xFF68), 0x40);

    bus.write8(0xFF68, 0x80 | 0x3F);
    bus.write8(0xFF69, 0x12);

    // Auto-increment wraps 0x3F -> 0x00 while preserving auto-inc bit.
    assert_eq!(bus.read8(0xFF68), 0xC0);

    bus.write8(0xFF68, 0x3F);
    assert_eq!(bus.read8(0xFF69), 0x12);

    // With auto-increment off, index should remain unchanged.
    bus.write8(0xFF68, 0x04);
    bus.write8(0xFF69, 0xAB);
    assert_eq!(bus.read8(0xFF68), 0x44);
    assert_eq!(bus.read8(0xFF69), 0xAB);
}

#[test]
fn bgpi_bgpd_are_gated_in_dmg_mode() {
    let mut bus = setup_dmg_bus();

    assert_eq!(bus.read8(0xFF68), 0xFF);
    assert_eq!(bus.read8(0xFF69), 0xFF);

    bus.write8(0xFF68, 0x80);
    bus.write8(0xFF69, 0x55);

    assert_eq!(bus.read8(0xFF68), 0xFF);
    assert_eq!(bus.read8(0xFF69), 0xFF);
}

#[test]
fn cgb_obj_palette_registers_support_index_and_auto_increment() {
    let mut bus = setup_cgb_bus();

    assert_eq!(bus.read8(0xFF6A), 0x40);

    bus.write8(0xFF6A, 0x80 | 0x3F);
    bus.write8(0xFF6B, 0x34);

    // Auto-increment wraps 0x3F -> 0x00 while preserving auto-inc bit.
    assert_eq!(bus.read8(0xFF6A), 0xC0);

    bus.write8(0xFF6A, 0x3F);
    assert_eq!(bus.read8(0xFF6B), 0x34);

    // With auto-increment off, index should remain unchanged.
    bus.write8(0xFF6A, 0x02);
    bus.write8(0xFF6B, 0xCD);
    assert_eq!(bus.read8(0xFF6A), 0x42);
    assert_eq!(bus.read8(0xFF6B), 0xCD);
}

#[test]
fn obpi_obpd_are_gated_in_dmg_mode() {
    let mut bus = setup_dmg_bus();

    assert_eq!(bus.read8(0xFF6A), 0xFF);
    assert_eq!(bus.read8(0xFF6B), 0xFF);

    bus.write8(0xFF6A, 0x80);
    bus.write8(0xFF6B, 0x66);

    assert_eq!(bus.read8(0xFF6A), 0xFF);
    assert_eq!(bus.read8(0xFF6B), 0xFF);
}

#[test]
fn cgb_bg_tile_attribute_bank_select_changes_fetched_tile_data() {
    let mut bus = setup_cgb_bus();

    // BG map entry uses tile 1.
    bus.vram[0x1800] = 1;
    // Attribute map in bank 1: select tile data bank 1.
    bus.vram[0x2000 + 0x1800] = 0x08;

    // Tile 1 row 0 in bank 0 => color 0.
    bus.vram[16] = 0x00;
    bus.vram[17] = 0x00;
    // Tile 1 row 0 in bank 1 => color 3.
    bus.vram[0x2000 + 16] = 0xFF;
    bus.vram[0x2000 + 17] = 0xFF;

    // Palette 0: color0 = white, color3 = red (BGR15: r=31,g=0,b=0 => 0x001F).
    write_bg_palette_color(&mut bus, 0, 0, 0x7FFF);
    write_bg_palette_color(&mut bus, 0, 3, 0x001F);

    bus.write8(0xFF40, 0x91);

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], 0xFFFF_0000);
}

#[test]
fn cgb_bg_priority_attribute_keeps_bg_over_sprite() {
    let mut bus = setup_cgb_bus();

    // BG tile in map at (0,0): tile 2.
    bus.vram[0x1800] = 2;
    // Attribute map in bank 1: BG-to-OAM priority set.
    bus.vram[0x2000 + 0x1800] = 0x80;

    // BG tile 2 row 0 => color 1 across the row.
    bus.vram[2 * 16] = 0xFF;
    bus.vram[2 * 16 + 1] = 0x00;

    // Sprite tile 1 row 0 => color 1 across the row.
    bus.vram[16] = 0xFF;
    bus.vram[17] = 0x00;

    // Sprite 0 at screen (0,0), above BG unless priority rules hide it.
    bus.oam[0] = 16;
    bus.oam[1] = 8;
    bus.oam[2] = 1;
    bus.oam[3] = 0x00;

    // Palette 0 color1 = green (BGR15: r=0,g=31,b=0 => 0x03E0).
    write_bg_palette_color(&mut bus, 0, 1, 0x03E0);

    bus.write8(0xFF47, 0xE4);
    bus.write8(0xFF48, 0xE4);
    bus.write8(0xFF40, 0x93); // LCD on, BG+OBJ enabled.

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], 0xFF00_FF00);
}

#[test]
fn cgb_bg_x_flip_attribute_flips_tile_pixels() {
    let mut bus = setup_cgb_bus();

    // BG map entry uses tile 4.
    bus.vram[0x1800] = 4;
    // Attribute map in bank 1: X flip.
    bus.vram[0x2000 + 0x1800] = 0x20;

    // Tile 4 row 0: leftmost pixel color 1, others color 0.
    bus.vram[4 * 16] = 0x80;
    bus.vram[4 * 16 + 1] = 0x00;

    // Palette 0 color0 = white, color1 = blue (BGR15: r=0,g=0,b=31 => 0x7C00).
    write_bg_palette_color(&mut bus, 0, 0, 0x7FFF);
    write_bg_palette_color(&mut bus, 0, 1, 0x7C00);

    bus.write8(0xFF40, 0x91);

    bus.tick(0);
    bus.tick(252);

    // Flipped horizontally: pixel appears at x=7 instead of x=0.
    assert_eq!(bus.ppu.framebuffer()[0], 0xFFFF_FFFF);
    assert_eq!(bus.ppu.framebuffer()[7], 0xFF00_00FF);
}

#[test]
fn cgb_sprite_uses_obj_palette_index_for_color() {
    let mut bus = setup_cgb_bus();

    // Sprite tile 1 row 0 => color 1 across the row.
    bus.vram[16] = 0xFF;
    bus.vram[17] = 0x00;

    // Sprite 0 at screen (0,0), attrs palette index = 3.
    bus.oam[0] = 16;
    bus.oam[1] = 8;
    bus.oam[2] = 1;
    bus.oam[3] = 0x03;

    // OBJ palette 3 color 1 = blue (BGR15: 0x7C00).
    write_obj_palette_color(&mut bus, 3, 1, 0x7C00);

    bus.write8(0xFF40, 0x93); // LCD on, BG+OBJ enabled.

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], 0xFF00_00FF);
}

#[test]
fn cgb_sprite_attribute_tile_bank_select_uses_vram_bank_1() {
    let mut bus = setup_cgb_bus();

    // Sprite tile 1 row 0 in bank 0 => color 0.
    bus.vram[16] = 0x00;
    bus.vram[17] = 0x00;
    // Sprite tile 1 row 0 in bank 1 => color 1.
    bus.vram[0x2000 + 16] = 0xFF;
    bus.vram[0x2000 + 17] = 0x00;

    // Sprite 0 at screen (0,0), attrs: bank1 + palette 1.
    bus.oam[0] = 16;
    bus.oam[1] = 8;
    bus.oam[2] = 1;
    bus.oam[3] = 0x08 | 0x01;

    // OBJ palette 1 color 1 = green.
    write_obj_palette_color(&mut bus, 1, 1, 0x03E0);

    bus.write8(0xFF40, 0x93);

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], 0xFF00_FF00);
}

#[test]
fn cgb_sprite_overlap_uses_oam_order_priority() {
    let mut bus = setup_cgb_bus();

    // Tile 1 row 0 => color 1 across row.
    bus.vram[16] = 0xFF;
    bus.vram[17] = 0x00;

    // Sprite 0 at x=8 (screen x=0), palette 1 => green.
    bus.oam[0] = 16;
    bus.oam[1] = 8;
    bus.oam[2] = 1;
    bus.oam[3] = 0x01;

    // Sprite 1 overlaps at same x, palette 2 => blue.
    bus.oam[4] = 16;
    bus.oam[5] = 8;
    bus.oam[6] = 1;
    bus.oam[7] = 0x02;

    write_obj_palette_color(&mut bus, 1, 1, 0x03E0);
    write_obj_palette_color(&mut bus, 2, 1, 0x7C00);

    bus.write8(0xFF40, 0x93);

    bus.tick(0);
    bus.tick(252);

    // CGB priority should pick earlier OAM entry (sprite 0, green).
    assert_eq!(bus.ppu.framebuffer()[0], 0xFF00_FF00);
}

#[test]
fn cgb_window_overrides_background_when_enabled() {
    let mut bus = setup_cgb_bus();

    // BG map (0x9800) at (0,0) uses tile 1.
    bus.vram[0x1800] = 1;
    // Window map (0x9C00) at (0,0) uses tile 2.
    bus.vram[0x1C00] = 2;

    // Tile 1 row 0 => color 1 across row.
    bus.vram[16] = 0xFF;
    bus.vram[17] = 0x00;
    // Tile 2 row 0 => color 2 across row.
    bus.vram[2 * 16] = 0x00;
    bus.vram[2 * 16 + 1] = 0xFF;

    // Palette 0: color1 = green, color2 = blue.
    write_bg_palette_color(&mut bus, 0, 1, 0x03E0);
    write_bg_palette_color(&mut bus, 0, 2, 0x7C00);

    // Enable LCD+BG+Window, window map 0x9C00, unsigned tile data.
    bus.write8(0xFF4A, 0x00); // WY
    bus.write8(0xFF4B, 0x07); // WX => window starts at x=0
    bus.write8(0xFF40, 0xF1);

    bus.tick(0);
    bus.tick(252);

    // Window pixel should override BG pixel at x=0.
    assert_eq!(bus.ppu.framebuffer()[0], 0xFF00_00FF);
}

#[test]
fn cgb_lcdc_bit0_zero_ignores_priorities_but_bg_remains_visible() {
    let mut bus = setup_cgb_bus();

    // BG tile in map at (0,0) and (1,0): tile 2.
    bus.vram[0x1800] = 2;
    bus.vram[0x1801] = 2;
    // Attribute map in bank 1: BG-to-OAM priority set.
    bus.vram[0x2000 + 0x1800] = 0x80;
    bus.vram[0x2000 + 0x1801] = 0x80;

    // BG tile 2 row 0 => color 1 across the row.
    bus.vram[2 * 16] = 0xFF;
    bus.vram[2 * 16 + 1] = 0x00;

    // Sprite tile 1 row 0 => color 1 across the row.
    bus.vram[16] = 0xFF;
    bus.vram[17] = 0x00;

    // Sprite 0 at screen (0,0)
    bus.oam[0] = 16;
    bus.oam[1] = 8;
    bus.oam[2] = 1;
    bus.oam[3] = 0x00; // Sprite-to-BG priority NOT set (sprite over BG).

    // Palette 0 color1 = green (BGR15: r=0,g=31,b=0 => 0x03E0).
    write_bg_palette_color(&mut bus, 0, 1, 0x03E0);
    // OBJ palette 0 color1 = red (BGR15: r=31,g=0,b=0 => 0x001F).
    write_obj_palette_color(&mut bus, 0, 1, 0x001F);

    // LCD on, OBJ enabled, BUT BG/Window priority bit 0 is OFF.
    // In CGB, this should mean BG is still visible, but priority bits are ignored.
    // So even though BG-to-OAM priority is set in VRAM, the sprite should appear.
    bus.write8(0xFF40, 0x92); // 1001 0010: LCD on, OBJ on, BG off (priority bit 0).

    bus.tick(0);
    bus.tick(252);

    // Sprite appears at x=0..7. Let's check x=0 (sprite) and x=8 (no sprite).
    // In CGB, BG should be visible even if LCDC bit 0 is 0.
    // x=0: Sprite (red)
    assert_eq!(bus.ppu.framebuffer()[0], 0xFFFF_0000);
    // x=8: BG color 1 (green)
    assert_eq!(bus.ppu.framebuffer()[8], 0xFF00_FF00);
}
