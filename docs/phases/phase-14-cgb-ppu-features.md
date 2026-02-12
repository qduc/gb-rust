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
  - CGB OBJ palette RAM (`FF6A/FF6B`) and CGB OBJ color conversion
  - Full CGB OBJ tile-bank behavior and CGB sprite palette index path
  - Cycle-exact CGB PPU fetch timing/priority edge cases

## Acceptance Criteria (first slice)
- [x] BG/window tile attributes are read from tilemap attribute bytes in VRAM bank 1.
- [x] Implemented CGB BG attribute bits: palette index (0..7), tile bank select, X/Y flip, BG-to-OAM priority.
- [x] Implemented CGB BG palette RAM register semantics for `FF68/FF69` including index and write auto-increment.
- [x] Render path converts CGB 15-bit BGR colors to 32-bit framebuffer output.
- [x] DMG behavior remains unchanged when not in CGB mode.
- [x] Focused tests validate palette register semantics, tile-bank select behavior, and priority/flip behavior.

## Tests Added
- [x] `cgb_bg_palette_registers_support_index_and_auto_increment`
- [x] `bgpi_bgpd_are_gated_in_dmg_mode`
- [x] `cgb_bg_tile_attribute_bank_select_changes_fetched_tile_data`
- [x] `cgb_bg_priority_attribute_keeps_bg_over_sprite`
- [x] `cgb_bg_x_flip_attribute_flips_tile_pixels`

## Implementation Notes
- `Ppu` now owns CGB BG palette RAM state and register behavior for `FF68/FF69`.
- `Bus` now routes `FF68/FF69` reads/writes to `Ppu` in CGB mode and gates them in DMG mode.
- Render path now has a CGB-aware entry point used by `Bus` that:
  - reads BG/window attributes from VRAM bank 1 tilemap region,
  - applies bank selection and X/Y flipping when fetching tile pixels,
  - resolves BG color via CGB BG palette RAM,
  - applies BG-to-OAM priority bit in sprite resolve path.
- Existing DMG render entry points are preserved for compatibility and tests.

## Exit Gate
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
