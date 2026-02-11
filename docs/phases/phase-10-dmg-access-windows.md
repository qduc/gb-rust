# Phase 10 (Milestone A): DMG Access Windows

## Status: COMPLETE

## Scope
- Files/modules to change:
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/tests/ppu_access_windows.rs` (new)
  - `docs/phases/phase-10-dmg-access-windows.md`
- Out of scope:
  - Hardware-exact OAM corruption behavior (`oam_bug`) internals
  - HDMA/GDMA and CGB-specific memory model work

## Acceptance Criteria
- [x] CPU reads/writes to VRAM (`0x8000..=0x9FFF`) are blocked during LCD mode 3 when LCD is enabled.
- [x] CPU reads/writes to OAM (`0xFE00..=0xFE9F`) are blocked during LCD modes 2 and 3 when LCD is enabled.
- [x] Outside blocked windows, accesses behave normally.
- [x] Add integration tests covering mode-boundary behavior.

## Tests
- [x] Add integration tests for VRAM mode-3 blocking and OAM mode-2/3 blocking.
- [x] Command: `cargo test --workspace`

## Implementation Steps
1. [x] Add phase doc and lock exact scope.
2. [x] Implement CPU access-window checks in `Bus::read8/write8` keyed by LCD enable + STAT mode.
3. [x] Add integration tests for blocked and unblocked windows.
4. [x] Run fmt/clippy/tests and update status.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Notes
- Added CPU access gating in `Bus` for DMG-compatible PPU windows:
  - VRAM blocked in mode 3 while LCD is enabled.
  - OAM blocked in modes 2 and 3 while LCD is enabled.
- Added regression coverage in `crates/gb-core/tests/ppu_access_windows.rs`.
