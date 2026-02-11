use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

fn make_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0148] = 0x00; // 32KB
    rom
}

fn setup_bus() -> Bus {
    let cart = Cartridge::from_rom(make_rom()).unwrap();
    Bus::new(cart)
}

const WHITE: u32 = 0xFFFF_FFFF;
const LIGHT: u32 = 0xFFAA_AAAA;
const DARK: u32 = 0xFF55_5555;
const BLACK: u32 = 0xFF00_0000;

fn write_tile_row(vram: &mut [u8; 0x2000], tile: u8, row: u8, lo: u8, hi: u8) {
    let base = (tile as usize) * 16 + (row as usize) * 2;
    vram[base] = lo;
    vram[base + 1] = hi;
}

#[test]
fn bg_renders_tile_pixels_unsigned_tiledata() {
    let mut bus = setup_bus();

    // Tile 1, row 0: color nums 0,1,2,3,0,1,2,3.
    write_tile_row(&mut bus.vram, 1, 0, 0x55, 0x33);
    bus.vram[0x1800] = 1; // BG map (0x9800) tile (0,0)

    bus.write8(0xFF47, 0xE4); // BGP identity
    bus.write8(0xFF40, 0x91); // LCD on, BG on, unsigned tile data, 0x9800 map

    bus.tick(0);
    bus.tick(252);

    assert_eq!(
        &bus.ppu.framebuffer()[0..8],
        &[WHITE, LIGHT, DARK, BLACK, WHITE, LIGHT, DARK, BLACK]
    );
}

#[test]
fn bg_scx_scrolls_horizontally() {
    let mut bus = setup_bus();

    write_tile_row(&mut bus.vram, 1, 0, 0x55, 0x33);
    bus.vram[0x1800] = 1;

    bus.write8(0xFF43, 4); // SCX
    bus.write8(0xFF47, 0xE4);
    bus.write8(0xFF40, 0x91);

    bus.tick(0);
    bus.tick(252);

    assert_eq!(&bus.ppu.framebuffer()[0..4], &[WHITE, LIGHT, DARK, BLACK]);
}

#[test]
fn bg_scy_scrolls_vertically() {
    let mut bus = setup_bus();

    // Tile 2: all pixels color 3 (black with identity palette).
    for row in 0..8u8 {
        write_tile_row(&mut bus.vram, 2, row, 0xFF, 0xFF);
    }
    // BG map row 1, col 0.
    bus.vram[0x1800 + 32] = 2;

    bus.write8(0xFF42, 8); // SCY (scroll down 1 tile)
    bus.write8(0xFF47, 0xE4);
    bus.write8(0xFF40, 0x91);

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], BLACK);
}

#[test]
fn bg_signed_tiledata_addressing_uses_0x9000_base() {
    let mut bus = setup_bus();

    // LCDC bit 4 = 0 => signed tile IDs, base effectively 0x9000.
    // Tile id -1 (0xFF) starts at 0x8FF0 => VRAM offset 0x0FF0.
    bus.vram[0x0FF0] = 0xFF;
    bus.vram[0x0FF1] = 0xFF;
    bus.vram[0x1800] = 0xFF;

    bus.write8(0xFF47, 0xE4);
    bus.write8(0xFF40, 0x81); // LCD on, BG on, signed tile data, 0x9800 map

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], BLACK);
}

#[test]
fn bg_bgp_palette_maps_color_numbers_to_shades() {
    let mut bus = setup_bus();

    // Tile 1, first pixel color num 1.
    write_tile_row(&mut bus.vram, 1, 0, 0x80, 0x00);
    bus.vram[0x1800] = 1;

    bus.write8(0xFF47, 0x1B); // invert mapping: 0->3, 1->2, 2->1, 3->0
    bus.write8(0xFF40, 0x91);

    bus.tick(0);
    bus.tick(252);

    assert_eq!(bus.ppu.framebuffer()[0], DARK); // color 1 -> shade 2
    assert_eq!(bus.ppu.framebuffer()[1], BLACK); // color 0 -> shade 3
}
