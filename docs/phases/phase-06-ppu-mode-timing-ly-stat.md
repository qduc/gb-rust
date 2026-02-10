# Phase 6: PPU — Mode Timing + LY/STAT

## Status: COMPLETE ✓

## Summary
Implemented a minimal DMG PPU timing state machine (mode 2/3/0 and VBlank mode 1), LY/LYC/STAT behavior, and VBlank/STAT interrupt requests. Added unit + integration tests for mode timing, coincidence edge interrupts, STAT write masking, and LY reset on write.

## Scope
- Files/modules to change:
  - `crates/gb-core/src/ppu/ppu.rs`
  - `crates/gb-core/src/bus/bus.rs`
  - (tests) `crates/gb-core/src/ppu/ppu.rs` (`#[cfg(test)]`) and/or `crates/gb-core/tests/ppu_timing.rs`
- Out of scope:
  - Background/window/sprite rendering
  - Accurate per-dot pixel pipeline
  - LCD register side-effects beyond what’s needed for LY/STAT + interrupts

## Acceptance Criteria
- [ ] PPU advances through modes 2/3/0 during visible scanlines and mode 1 during VBlank with correct per-scanline timing.
- [ ] LY (FF44) increments at the correct cadence, enters VBlank at LY=144, and wraps after LY=153.
- [ ] STAT (FF41) mode bits (0-1) and coincidence bit (2) reflect current state.
- [ ] VBlank interrupt is requested at VBlank entry; STAT interrupt is requested when enabled on relevant edges.

## Tests
- Unit tests:
  - PPU mode transition timing (dot/cycle thresholds)
  - LY increment/reset and LYC coincidence
  - Interrupt request behavior for VBlank + STAT
- Command to run: `cargo test --workspace`

## Implementation Steps
1. Add failing tests covering mode timing + LY/STAT semantics.
2. Implement minimal PPU timing state machine in `Ppu::tick()`.
3. Wire PPU register behavior via `Bus` IO mapping (STAT write mask, LY write reset).
4. Add interrupt request edge detection for STAT sources and VBlank.
5. Run full gates and update `docs/roadmap.md`.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Notes
- Timing targets (DMG): 456 dots/line; visible lines 0–143; VBlank lines 144–153.
- Mode timing (simplified, DMG): Mode 2 (OAM) = 80 dots; Mode 3 (transfer) = 172+ dots (variable on real HW); Mode 0 (HBlank) = remainder of 456.
- STAT coincidence (bit 2) is set when `LY == LYC`; the STAT coincidence interrupt should fire on the *rising edge* (false→true), not continuously.
