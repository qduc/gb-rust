//! Pixel rendering helpers (DMG).

use super::{Framebuffer, LCD_HEIGHT, LCD_WIDTH};

pub const DMG_SHADES: [u32; 4] = [0xFFFF_FFFF, 0xFFAA_AAAA, 0xFF55_5555, 0xFF00_0000];

const LCDC: usize = 0x40;
const SCY: usize = 0x42;
const SCX: usize = 0x43;
const BGP: usize = 0x47;
const OBP0: usize = 0x48;
const OBP1: usize = 0x49;
const WY: usize = 0x4A;
const WX: usize = 0x4B;

#[derive(Copy, Clone, Default)]
struct BgPixelInfo {
    color_num: u8,
    bg_to_oam_priority: bool,
}

fn scale_5bit_to_8bit(v: u8) -> u8 {
    (v << 3) | (v >> 2)
}

fn cgb_bgr15_to_argb(color: u16) -> u32 {
    let b = (color & 0x1F) as u8;
    let g = ((color >> 5) & 0x1F) as u8;
    let r = ((color >> 10) & 0x1F) as u8;

    let r8 = scale_5bit_to_8bit(r);
    let g8 = scale_5bit_to_8bit(g);
    let b8 = scale_5bit_to_8bit(b);

    0xFF00_0000 | ((r8 as u32) << 16) | ((g8 as u32) << 8) | (b8 as u32)
}

fn cgb_bg_color(bg_palette_ram: &[u8; 0x40], palette: u8, color_num: u8) -> u32 {
    let base = (palette as usize) * 8 + (color_num as usize) * 2;
    let lo = bg_palette_ram[base];
    let hi = bg_palette_ram[base + 1];
    let color = u16::from_le_bytes([lo, hi]);
    cgb_bgr15_to_argb(color)
}

fn cgb_obj_color(obj_palette_ram: &[u8; 0x40], palette: u8, color_num: u8) -> u32 {
    let base = (palette as usize) * 8 + (color_num as usize) * 2;
    let lo = obj_palette_ram[base];
    let hi = obj_palette_ram[base + 1];
    let color = u16::from_le_bytes([lo, hi]);
    cgb_bgr15_to_argb(color)
}

#[allow(clippy::too_many_arguments)]
fn render_bg_window_scanline(
    framebuffer: &mut Framebuffer,
    ly: u8,
    vram0: &[u8; 0x2000],
    vram1: Option<&[u8; 0x2000]>,
    io: &[u8; 0x80],
    cgb_mode: bool,
    bg_palette_ram: &[u8; 0x40],
    mut bg_pixels: Option<&mut [BgPixelInfo; LCD_WIDTH]>,
) {
    if ly as usize >= LCD_HEIGHT {
        return;
    }

    let lcdc = io[LCDC];
    // On DMG, bit 0 controls both BG and window rendering.
    let bg_enabled = (lcdc & 0x01) != 0;
    let window_enabled = bg_enabled && (lcdc & 0x20) != 0;

    let scy = io[SCY];
    let scx = io[SCX];
    let bgp = io[BGP];

    let bg_tilemap_base = if (lcdc & 0x08) != 0 { 0x9C00 } else { 0x9800 };
    let window_tilemap_base = if (lcdc & 0x40) != 0 { 0x9C00 } else { 0x9800 };
    let tiledata_unsigned = (lcdc & 0x10) != 0;

    // Background coordinates (wrap around 256x256).
    let y = ly.wrapping_add(scy);
    let bg_tile_row = y as u16 / 8;
    let bg_pixel_row = y as u16 % 8;

    // Window coordinates (no scroll); visible when LY >= WY and X >= WX-7.
    let wy = io[WY];
    let wx = io[WX];
    let window_active_line = window_enabled && ly >= wy;
    let window_y = ly.wrapping_sub(wy) as u16;
    let win_tile_row = window_y / 8;
    let win_pixel_row = window_y % 8;
    let win_x_start = (wx as i16) - 7;

    for x in 0..(LCD_WIDTH as u16) {
        let mut color_num = 0u8;
        let mut cgb_pixel_written = false;

        if bg_enabled {
            let bx = (x as u8).wrapping_add(scx);
            let bg_tile_col = bx as u16 / 8;
            let bg_pixel_col = bx as u16 % 8;

            let tilemap_addr = bg_tilemap_base + bg_tile_row * 32 + bg_tile_col;
            let tilemap_off = (tilemap_addr - 0x8000) as usize;
            let tile_id = vram0[tilemap_off];
            let attrs = if cgb_mode {
                vram1.map_or(0, |bank1| bank1[tilemap_off])
            } else {
                0
            };
            let tile_bank = if (attrs & 0x08) != 0 { 1 } else { 0 };
            let palette_num = attrs & 0x07;
            let y_flip = (attrs & 0x40) != 0;
            let x_flip = (attrs & 0x20) != 0;
            let bg_to_oam_priority = (attrs & 0x80) != 0;

            let mut pixel_row = bg_pixel_row;
            if y_flip {
                pixel_row = 7 - pixel_row;
            }

            let mut pixel_col = bg_pixel_col as u8;
            if x_flip {
                pixel_col = 7 - pixel_col;
            }

            let tile_addr = if tiledata_unsigned {
                0x8000u16 + (tile_id as u16) * 16
            } else {
                // Signed tile IDs index into 0x8800..=0x97FF, with tile 0 at 0x9000.
                let id = tile_id as i8 as i16;
                (0x9000i32 + (id as i32) * 16) as u16
            };

            let row_addr = tile_addr + pixel_row * 2;
            let tile_vram = if cgb_mode && tile_bank == 1 {
                vram1.unwrap_or(vram0)
            } else {
                vram0
            };
            let lo = tile_vram[(row_addr - 0x8000) as usize];
            let hi = tile_vram[(row_addr - 0x8000 + 1) as usize];
            let bit = 7 - pixel_col;
            let lsb = (lo >> bit) & 1;
            let msb = (hi >> bit) & 1;
            color_num = (msb << 1) | lsb;

            if let Some(ref mut px) = bg_pixels {
                px[x as usize].bg_to_oam_priority = bg_to_oam_priority;
                px[x as usize].color_num = color_num;
            }

            if cgb_mode {
                framebuffer[(ly as usize) * LCD_WIDTH + (x as usize)] =
                    cgb_bg_color(bg_palette_ram, palette_num, color_num);
                cgb_pixel_written = true;
            }
        }

        if window_active_line && (x as i16) >= win_x_start {
            let win_x = (x as i16 - win_x_start) as u16;
            let win_tile_col = win_x / 8;
            let win_pixel_col = win_x % 8;

            let tilemap_addr = window_tilemap_base + win_tile_row * 32 + win_tile_col;
            let tilemap_off = (tilemap_addr - 0x8000) as usize;
            let tile_id = vram0[tilemap_off];
            let attrs = if cgb_mode {
                vram1.map_or(0, |bank1| bank1[tilemap_off])
            } else {
                0
            };
            let tile_bank = if (attrs & 0x08) != 0 { 1 } else { 0 };
            let palette_num = attrs & 0x07;
            let y_flip = (attrs & 0x40) != 0;
            let x_flip = (attrs & 0x20) != 0;
            let bg_to_oam_priority = (attrs & 0x80) != 0;

            let mut pixel_row = win_pixel_row;
            if y_flip {
                pixel_row = 7 - pixel_row;
            }

            let mut pixel_col = win_pixel_col as u8;
            if x_flip {
                pixel_col = 7 - pixel_col;
            }

            let tile_addr = if tiledata_unsigned {
                0x8000u16 + (tile_id as u16) * 16
            } else {
                let id = tile_id as i8 as i16;
                (0x9000i32 + (id as i32) * 16) as u16
            };

            let row_addr = tile_addr + pixel_row * 2;
            let tile_vram = if cgb_mode && tile_bank == 1 {
                vram1.unwrap_or(vram0)
            } else {
                vram0
            };
            let lo = tile_vram[(row_addr - 0x8000) as usize];
            let hi = tile_vram[(row_addr - 0x8000 + 1) as usize];
            let bit = 7 - pixel_col;
            let lsb = (lo >> bit) & 1;
            let msb = (hi >> bit) & 1;
            color_num = (msb << 1) | lsb;

            if let Some(ref mut px) = bg_pixels {
                px[x as usize].bg_to_oam_priority = bg_to_oam_priority;
                px[x as usize].color_num = color_num;
            }

            if cgb_mode {
                framebuffer[(ly as usize) * LCD_WIDTH + (x as usize)] =
                    cgb_bg_color(bg_palette_ram, palette_num, color_num);
                cgb_pixel_written = true;
            }
        }

        if cgb_mode && cgb_pixel_written {
            continue;
        }

        if let Some(ref mut px) = bg_pixels {
            px[x as usize].color_num = color_num;
        }

        let shade = (bgp >> (color_num * 2)) & 0x03;
        framebuffer[(ly as usize) * LCD_WIDTH + (x as usize)] = DMG_SHADES[shade as usize];
    }
}

pub fn render_bg_scanline(
    framebuffer: &mut Framebuffer,
    ly: u8,
    vram: &[u8; 0x2000],
    io: &[u8; 0x80],
) {
    render_bg_window_scanline(framebuffer, ly, vram, None, io, false, &[0; 0x40], None);
}

#[derive(Copy, Clone)]
struct SpriteLine {
    oam_index: u8,
    x: i16,
    attrs: u8,
    row_lo: u8,
    row_hi: u8,
}

#[allow(clippy::too_many_arguments)]
fn render_obj_scanline(
    framebuffer: &mut Framebuffer,
    ly: u8,
    vram0: &[u8; 0x2000],
    vram1: Option<&[u8; 0x2000]>,
    oam: &[u8; 0xA0],
    io: &[u8; 0x80],
    cgb_mode: bool,
    bg_pixels: &[BgPixelInfo; LCD_WIDTH],
    obj_palette_ram: &[u8; 0x40],
) {
    if ly as usize >= LCD_HEIGHT {
        return;
    }

    let lcdc = io[LCDC];
    let sprites_enabled = (lcdc & 0x02) != 0;
    if !sprites_enabled {
        return;
    }

    let sprite_height: i16 = if (lcdc & 0x04) != 0 { 16 } else { 8 };
    let ly_i16 = ly as i16;

    let mut line_sprites: [SpriteLine; 10] = [SpriteLine {
        oam_index: 0,
        x: 0,
        attrs: 0,
        row_lo: 0,
        row_hi: 0,
    }; 10];
    let mut count = 0usize;

    for i in 0..40u8 {
        let base = (i as usize) * 4;
        let y = (oam[base] as i16) - 16;
        let x = (oam[base + 1] as i16) - 8;
        let mut tile = oam[base + 2];
        let attrs = oam[base + 3];

        if ly_i16 < y || ly_i16 >= y + sprite_height {
            continue;
        }

        let y_flip = (attrs & 0x40) != 0;
        let mut row = ly_i16 - y;
        if y_flip {
            row = sprite_height - 1 - row;
        }

        if sprite_height == 16 {
            tile &= 0xFE;
            if row >= 8 {
                tile = tile.wrapping_add(1);
                row -= 8;
            }
        }

        let tile_addr = 0x8000u16 + (tile as u16) * 16;
        let row_addr = tile_addr + (row as u16) * 2;
        let tile_vram = if cgb_mode && (attrs & 0x08) != 0 {
            vram1.unwrap_or(vram0)
        } else {
            vram0
        };
        let row_lo = tile_vram[(row_addr - 0x8000) as usize];
        let row_hi = tile_vram[(row_addr - 0x8000 + 1) as usize];

        line_sprites[count] = SpriteLine {
            oam_index: i,
            x,
            attrs,
            row_lo,
            row_hi,
        };
        count += 1;
        if count == 10 {
            break;
        }
    }

    let obp0 = io[OBP0];
    let obp1 = io[OBP1];
    let bg_enabled = (lcdc & 0x01) != 0;

    for x in 0..LCD_WIDTH {
        let screen_x = x as i16;

        let mut best: Option<(i16, u8, u8, u8)> = None;
        // (sprite_x, oam_index, attrs, color_num)

        for sprite in &line_sprites[..count] {
            if screen_x < sprite.x || screen_x >= sprite.x + 8 {
                continue;
            }

            let mut col = (screen_x - sprite.x) as u8;
            let x_flip = (sprite.attrs & 0x20) != 0;
            if x_flip {
                col = 7 - col;
            }
            let bit = 7 - col;
            let lsb = (sprite.row_lo >> bit) & 1;
            let msb = (sprite.row_hi >> bit) & 1;
            let color_num = (msb << 1) | lsb;
            if color_num == 0 {
                continue;
            }

            if cgb_mode {
                best = Some((sprite.x, sprite.oam_index, sprite.attrs, color_num));
                break;
            }

            let key = (sprite.x, sprite.oam_index);
            match best {
                None => best = Some((key.0, key.1, sprite.attrs, color_num)),
                Some((best_x, best_i, _, _)) => {
                    if key < (best_x, best_i) {
                        best = Some((key.0, key.1, sprite.attrs, color_num));
                    }
                }
            }
        }

        let Some((_, _, attrs, color_num)) = best else {
            continue;
        };

        let behind_bg = (attrs & 0x80) != 0;
        let bg_nonzero = bg_pixels[x].color_num != 0;

        if cgb_mode {
            if (behind_bg || bg_pixels[x].bg_to_oam_priority) && bg_nonzero {
                continue;
            }
        } else if behind_bg && bg_enabled && bg_nonzero {
            continue;
        }

        if cgb_mode {
            let palette_num = attrs & 0x07;
            framebuffer[(ly as usize) * LCD_WIDTH + x] =
                cgb_obj_color(obj_palette_ram, palette_num, color_num);
        } else {
            let use_obp1 = (attrs & 0x10) != 0;
            let pal = if use_obp1 { obp1 } else { obp0 };
            let shade = (pal >> (color_num * 2)) & 0x03;
            framebuffer[(ly as usize) * LCD_WIDTH + x] = DMG_SHADES[shade as usize];
        }
    }
}

pub fn render_scanline(
    framebuffer: &mut Framebuffer,
    ly: u8,
    vram: &[u8; 0x2000],
    oam: &[u8; 0xA0],
    io: &[u8; 0x80],
) {
    let mut bg_pixels = [BgPixelInfo::default(); LCD_WIDTH];
    render_bg_window_scanline(
        framebuffer,
        ly,
        vram,
        None,
        io,
        false,
        &[0; 0x40],
        Some(&mut bg_pixels),
    );
    render_obj_scanline(
        framebuffer,
        ly,
        vram,
        None,
        oam,
        io,
        false,
        &bg_pixels,
        &[0; 0x40],
    );
}

#[allow(clippy::too_many_arguments)]
pub fn render_scanline_with_cgb(
    framebuffer: &mut Framebuffer,
    ly: u8,
    vram0: &[u8; 0x2000],
    vram1: Option<&[u8; 0x2000]>,
    oam: &[u8; 0xA0],
    io: &[u8; 0x80],
    cgb_mode: bool,
    bg_palette_ram: &[u8; 0x40],
    obj_palette_ram: &[u8; 0x40],
) {
    let mut bg_pixels = [BgPixelInfo::default(); LCD_WIDTH];
    render_bg_window_scanline(
        framebuffer,
        ly,
        vram0,
        vram1,
        io,
        cgb_mode,
        bg_palette_ram,
        Some(&mut bg_pixels),
    );
    render_obj_scanline(
        framebuffer,
        ly,
        vram0,
        vram1,
        oam,
        io,
        cgb_mode,
        &bg_pixels,
        obj_palette_ram,
    );
}

#[cfg(test)]
mod tests {
    use super::{render_scanline, DMG_SHADES, LCD_WIDTH};

    const LCDC: usize = 0x40;
    const BGP: usize = 0x47;
    const OBP0: usize = 0x48;

    fn write_tile(vram: &mut [u8; 0x2000], tile: usize, rows: &[(u8, u8); 8]) {
        let base = tile * 16;
        for (r, (lo, hi)) in rows.iter().enumerate() {
            vram[base + r * 2] = *lo;
            vram[base + r * 2 + 1] = *hi;
        }
    }

    #[test]
    fn sprite_renders_over_bg_and_respects_transparency() {
        let mut fb = [0u32; 160 * 144];
        let mut vram = [0u8; 0x2000];
        let mut oam = [0u8; 0xA0];
        let mut io = [0u8; 0x80];

        // Tile 1: all pixels color 1 (lsb=1).
        write_tile(&mut vram, 1, &[(0xFF, 0x00); 8]);

        // Sprite 0 at (0,0), tile 1, above BG.
        oam[0] = 16;
        oam[1] = 8;
        oam[2] = 1;
        oam[3] = 0;

        io[BGP] = 0xE4;
        io[OBP0] = 0xE4;
        io[LCDC] = 0x93; // BG+OBJ enabled, 8x8 sprites, unsigned tile data

        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[1]);
    }

    #[test]
    fn sprite_priority_bit_hides_behind_nonzero_bg() {
        let mut fb = [0u32; 160 * 144];
        let mut vram = [0u8; 0x2000];
        let mut oam = [0u8; 0xA0];
        let mut io = [0u8; 0x80];

        // BG tile 2: color 3 (black).
        write_tile(&mut vram, 2, &[(0xFF, 0xFF); 8]);
        // BG map (0x9800): top-left tile is 2.
        vram[0x1800] = 2;

        // Sprite tile 1: color 1.
        write_tile(&mut vram, 1, &[(0xFF, 0x00); 8]);

        // Sprite behind BG.
        oam[0] = 16;
        oam[1] = 8;
        oam[2] = 1;
        oam[3] = 0x80;

        io[BGP] = 0xE4;
        io[OBP0] = 0xE4;
        io[LCDC] = 0x93;

        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[3]);

        // If BG color is 0, sprite should show even with priority bit set.
        vram[0x1800] = 0;
        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[1]);
    }

    #[test]
    fn sprite_x_and_y_flip() {
        let mut fb = [0u32; 160 * 144];
        let mut vram = [0u8; 0x2000];
        let mut oam = [0u8; 0xA0];
        let mut io = [0u8; 0x80];

        // Tile 3: leftmost pixel color 1, rightmost pixel color 2.
        write_tile(&mut vram, 3, &[(0x80, 0x01); 8]);

        // Sprite at (0,0), tile 3.
        oam[0] = 16;
        oam[1] = 8;
        oam[2] = 3;

        io[BGP] = 0xE4;
        io[OBP0] = 0xE4;
        io[LCDC] = 0x93;

        oam[3] = 0x00;
        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[1]);
        assert_eq!(fb[7], DMG_SHADES[2]);

        oam[3] = 0x20; // X flip
        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[2]);
        assert_eq!(fb[7], DMG_SHADES[1]);

        // Tile 4: top row color 1, bottom row color 2.
        let mut rows = [(0xFF, 0x00); 8];
        rows[7] = (0x00, 0xFF);
        write_tile(&mut vram, 4, &rows);
        oam[2] = 4;

        oam[3] = 0x00;
        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[1]);

        oam[3] = 0x40; // Y flip
        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[2]);
    }

    #[test]
    fn sprite_8x16_uses_two_tiles() {
        let mut fb = [0u32; 160 * 144];
        let mut vram = [0u8; 0x2000];
        let mut oam = [0u8; 0xA0];
        let mut io = [0u8; 0x80];

        write_tile(&mut vram, 6, &[(0xFF, 0x00); 8]);
        write_tile(&mut vram, 7, &[(0x00, 0xFF); 8]);

        oam[0] = 16;
        oam[1] = 8;
        oam[2] = 6;
        oam[3] = 0;

        io[BGP] = 0xE4;
        io[OBP0] = 0xE4;
        io[LCDC] = 0x97; // 8x16 sprites

        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[1]);

        render_scanline(&mut fb, 8, &vram, &oam, &io);
        assert_eq!(fb[8 * LCD_WIDTH], DMG_SHADES[2]);
    }

    #[test]
    fn sprite_per_line_limit_is_enforced() {
        let mut fb = [0u32; 160 * 144];
        let mut vram = [0u8; 0x2000];
        let mut oam = [0u8; 0xA0];
        let mut io = [0u8; 0x80];

        // Tile 1: color 1.
        write_tile(&mut vram, 1, &[(0xFF, 0x00); 8]);

        // First 10 sprites are fully transparent (tile 0 => color 0), 11th is visible.
        for i in 0..10 {
            let base = i * 4;
            oam[base] = 16;
            oam[base + 1] = 8;
            oam[base + 2] = 0;
            oam[base + 3] = 0;
        }
        let base = 10 * 4;
        oam[base] = 16;
        oam[base + 1] = 8;
        oam[base + 2] = 1;
        oam[base + 3] = 0;

        io[BGP] = 0xE4;
        io[OBP0] = 0xE4;
        io[LCDC] = 0x93;

        render_scanline(&mut fb, 0, &vram, &oam, &io);
        assert_eq!(fb[0], DMG_SHADES[0]);
    }
}
