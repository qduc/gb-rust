# Phase 13: CGB Memory Model

## Status: COMPLETE

## Scope
- Files/modules to change:
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/tests/cgb_memory.rs` (new)
  - `docs/phases/phase-13-cgb-memory-model.md`
- Out of scope:
  - CGB PPU attribute/palette behavior
  - CGB audio/timing adjustments

## Acceptance Criteria
- [x] CGB mode exposes `VBK` (`0xFF4F`) and switches CPU-visible VRAM bank for `0x8000..=0x9FFF`.
- [x] CGB mode exposes `SVBK` (`0xFF70`) and switches WRAM bank for `0xD000..=0xDFFF`, with bank 0 remapped to bank 1.
- [x] WRAM bank 0 at `0xC000..=0xCFFF` remains fixed regardless of `SVBK`.
- [x] DMG mode ignores `VBK`/`SVBK` writes and does not enable banked behavior.
- [x] CGB mode exposes `HDMA1..HDMA5` (`0xFF51..=0xFF55`) with DMG gating and masked register semantics.
- [x] GDMA (bit7=0 write to `HDMA5`) copies all requested 0x10-byte blocks immediately.
- [x] HDMA (bit7=1 write to `HDMA5`) copies one 0x10-byte block per HBlank and supports termination by writing bit7=0.

## Tests
- [x] Add integration tests for VRAM bank switching in CGB and DMG gating.
- [x] Add integration tests for WRAM banking (`SVBK`) semantics and fixed bank 0.
- [x] Add integration tests for `HDMA1..HDMA5` DMG gating.
- [x] Add integration tests for GDMA completion/status and HDMA per-HBlank stepping/termination.
- [x] Command: `cargo test --workspace`

## Implementation Steps
1. [x] Add phase doc and lock target file/test scope.
2. [x] Implement CGB banked VRAM and WRAM backing storage in `Bus`.
3. [x] Add `VBK`/`SVBK` IO register read/write behavior with DMG gating.
4. [x] Add integration tests for bank switching and register semantics.
5. [x] Implement CGB HDMA/GDMA registers (`FF51..FF55`) and transfer behavior.
6. [x] Add HDMA/GDMA integration tests.
7. [x] Run fmt/clippy/tests and update phase status.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Notes
- Implemented on 2026-02-11:
  - CGB `VBK` register semantics (`0xFF4F`) with DMG gating.
  - CGB `SVBK` register semantics (`0xFF70`) with bank 0 remap to bank 1.
  - 2-bank VRAM backing (`0x4000`) and 8-bank WRAM backing (`0x8000`) in `Bus`.
  - Integration tests: `crates/gb-core/tests/cgb_memory.rs`.
- Implemented on 2026-02-12:
  - CGB `HDMA1..HDMA5` (`0xFF51..=0xFF55`) register semantics with DMG gating and masked source/destination low nibble behavior.
  - GDMA immediate transfer behavior for `(N+1) * 0x10` bytes.
  - HDMA transfer behavior: one block per visible-line HBlank (mode 0), with write-to-`HDMA5` termination support.

### Limitations
- HDMA timing is pragmatic rather than cycle-perfect: transfer triggers once per entered HBlank line and does not model fine-grained bus stalls.
- When LCD is disabled while HDMA is active, remaining blocks are completed immediately for deterministic behavior in this codebase.
