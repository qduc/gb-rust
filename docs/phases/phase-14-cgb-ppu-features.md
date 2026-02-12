# Phase 14: CGB PPU Features

## Status: COMPLETED

## Scope
- Files/modules changed:
  - `crates/gb-core/src/ppu/ppu.rs`
  - `crates/gb-core/src/ppu/render.rs`
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/tests/cgb_ppu.rs`
  - `docs/phases/phase-14-cgb-ppu-features.md`
  - `docs/roadmap.md`

## Acceptance Criteria
- [x] BG/window tile attributes are read from tilemap attribute bytes in VRAM bank 1.
- [x] Implemented CGB BG attribute bits: palette index (0..7), tile bank select, X/Y flip, BG-to-OAM priority.
- [x] Implemented CGB BG palette RAM register semantics for `FF68/FF69` including index and write auto-increment.
- [x] Render path converts CGB 15-bit RGB colors (correctly mapped R/G/B) to 32-bit framebuffer output.
- [x] Implemented CGB OBJ palette RAM register semantics for `FF6A/FF6B` including index and write auto-increment.
- [x] CGB sprites use palette index bits (OAM attr bits 0..2) and resolve sprite colors via CGB OBJ palette RAM.
- [x] CGB sprites support VRAM tile bank select (OAM attr bit 3) when fetching sprite tile data.
- [x] Implemented CGB-specific priority rules:
  - [x] OAM order priority (lowest index wins).
  - [x] LCDC bit 0 as "master priority" flag: when 0, BG/Window are always visible but priority bits are ignored (sprites always on top).
- [x] DMG behavior remains unchanged when not in CGB mode.
- [x] Focused tests validate palette register semantics, tile-bank select behavior, priority/flip behavior, and LCDC bit 0 behavior.

## Tests Added
- [x] `cgb_bg_palette_registers_support_index_and_auto_increment`
- [x] `bgpi_bgpd_are_gated_in_dmg_mode`
- [x] `cgb_bg_tile_attribute_bank_select_changes_fetched_tile_data`
- [x] `cgb_bg_priority_attribute_keeps_bg_over_sprite`
- [x] `cgb_bg_x_flip_attribute_flips_tile_pixels`
- [x] `cgb_obj_palette_registers_support_index_and_auto_increment`
- [x] `obpi_obpd_are_gated_in_dmg_mode`
- [x] `cgb_sprite_uses_obj_palette_index_for_color`
- [x] `cgb_sprite_attribute_tile_bank_select_uses_vram_bank_1`
- [x] `cgb_sprite_overlap_uses_oam_order_priority`
- [x] `cgb_lcdc_bit0_zero_ignores_priorities_but_bg_remains_visible`

## Implementation Notes
- `Ppu` owns CGB BG/OBJ palette RAM and register semantics for `FF68-FF6B`.
- `Bus` handles routing and CGB gating for these registers.
- Render path in `render.rs` uses a CGB-aware path that:
  - reads BG/window attributes from VRAM bank 1,
  - respects bank selection, X/Y flipping, and priority attributes,
  - correctly maps CGB 15-bit RGB colors (low bits are Red, high bits are Blue).
- Sprite rendering in CGB mode uses OAM-order priority and VRAM bank 1 selection.
- LCDC bit 0 is treated as a master priority flag in CGB mode, ensuring sprites always appear over BG/Window if it's 0 (while keeping BG/Window visible).

## Remaining gaps
- Cycle-exact CGB PPU fetch timing (Mode 3 duration variations).
- Hardware-accurate OAM-to-pixel-pipeline timing edge cases.
- These are deferred to a later hardening phase.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
