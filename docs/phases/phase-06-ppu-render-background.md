# Phase 6: PPU — Render Background

## Status: IN PROGRESS

## Scope
- Files/modules to change:
  - `crates/gb-core/src/ppu/ppu.rs`
  - `crates/gb-core/src/bus/bus.rs` (only if needed to pass VRAM into PPU)
- Out of scope:
  - Window rendering
  - Sprite rendering
  - Cycle-accurate pixel FIFO / mode 3 variable length

## Acceptance Criteria
- [ ] When LCDC enables BG rendering, the PPU produces correct background pixels in the framebuffer using VRAM tile data + selected tile map.
- [ ] SCX/SCY scroll registers affect background selection.
- [ ] Tile data addressing works for both LCDC tile data modes (0x8000 unsigned and 0x8800 signed).
- [ ] BGP palette register maps 2-bit color indices to final grayscale values.

## Tests
- Unit tests:
  - Tile data decode (2bpp)
  - Tile indexing (signed/unsigned)
  - Scanline render with known VRAM patterns + scroll
- Command to run: `cargo test --workspace`

## Implementation Steps
1. Gather spec notes for BG tile addressing and palette mapping.
2. Add failing tests that render a known scanline into `Ppu::framebuffer`.
3. Wire VRAM access into PPU tick/render path (pass VRAM slice or provide accessor).
4. Render one background scanline per visible LY (simple fixed timing: render at mode3→mode0 boundary).
5. Run full gates and update `docs/roadmap.md`.

## Exit Gate
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Notes
- Real HW mode 3 length is variable; for now we keep the fixed 172-dot transfer from the timing phase.
