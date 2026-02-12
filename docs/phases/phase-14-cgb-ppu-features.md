# Phase 14: CGB PPU Features

## Status: IN PROGRESS (first slice landed)

## Scope
- Files/modules changed in this slice:
  - `crates/gb-core/src/ppu/ppu.rs`
  - `crates/gb-core/src/ppu/render.rs`
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/tests/cgb_ppu.rs` (new)
  - `docs/phases/phase-14-cgb-ppu-features.md`
  - `docs/roadmap.md`
- Out of scope for this slice:
  - Cycle-exact CGB PPU fetch timing/priority edge cases
  - Remaining CGB OBJ edge cases (see “Remaining gaps”)

## Acceptance Criteria (first slice)
- [x] BG/window tile attributes are read from tilemap attribute bytes in VRAM bank 1.
- [x] Implemented CGB BG attribute bits: palette index (0..7), tile bank select, X/Y flip, BG-to-OAM priority.
- [x] Implemented CGB BG palette RAM register semantics for `FF68/FF69` including index and write auto-increment.
- [x] Render path converts CGB 15-bit BGR colors to 32-bit framebuffer output.
- [x] Implemented CGB OBJ palette RAM register semantics for `FF6A/FF6B` including index and write auto-increment.
- [x] CGB sprites use palette index bits (OAM attr bits 0..2) and resolve sprite colors via CGB OBJ palette RAM.
- [x] CGB sprites support VRAM tile bank select (OAM attr bit 3) when fetching sprite tile data.
- [x] DMG behavior remains unchanged when not in CGB mode.
- [x] Focused tests validate palette register semantics, tile-bank select behavior, and priority/flip behavior.

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

## Implementation Notes
- `Ppu` now owns CGB BG palette RAM state and register behavior for `FF68/FF69`.
- `Ppu` now also owns CGB OBJ palette RAM state and register behavior for `FF6A/FF6B`.
- `Bus` now routes `FF68/FF69` reads/writes to `Ppu` in CGB mode and gates them in DMG mode.
- `Bus` now routes `FF6A/FF6B` reads/writes to `Ppu` in CGB mode and gates them in DMG mode.
- Render path now has a CGB-aware entry point used by `Bus` that:
  - reads BG/window attributes from VRAM bank 1 tilemap region,
  - applies bank selection and X/Y flipping when fetching tile pixels,
  - resolves BG color via CGB BG palette RAM,
  - applies BG-to-OAM priority bit in sprite resolve path.
- Sprite resolve path in CGB mode additionally:
  - fetches sprite tile data from VRAM bank 1 when OAM attr bit 3 is set,
  - resolves sprite color via CGB OBJ palette RAM using palette index bits 0..2.
- Existing DMG render entry points are preserved for compatibility and tests.

## Remaining gaps
- CGB sprite overlap priority is implemented with OAM-order preference (vs DMG X-then-OAM tie-break). Further hardware-accurate edge cases (including some interactions with sprite X sorting under per-line limits) are still deferred.
- CGB OBJ “master priority” and other LCDC/attribute corner cases are not cycle-accurate.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
