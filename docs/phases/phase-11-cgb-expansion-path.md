# Phase 11: CGB Expansion Path (Post-DMG Milestone)

## Scope
- Files/modules to change (planning targets):
  - `crates/gb-core/src/gb.rs`
  - `crates/gb-core/src/bus.rs`
  - `crates/gb-core/src/cpu/`
  - `crates/gb-core/src/ppu/`
  - `crates/gb-core/src/apu/`
  - `crates/gb-core/tests/`
  - `crates/gb-cli/src/main.rs`
- Out of scope:
  - New non-CGB frontend features unrelated to correctness
  - Performance optimizations not required for correctness

## Acceptance Criteria
- [ ] DMG milestone exit gate is complete before any CGB implementation starts.
- [ ] CGB work is split into ordered phases with per-phase ROM evidence.
- [ ] CGB ROM manifest/status table is kept up to date in docs during implementation.

## Tests
- Unit tests:
  - KEY1/speed-switch register behavior
  - CGB-only register read/write gating
  - VRAM/WRAM bank selection behavior
  - CGB palette/attribute decode and priority rules
- Integration tests:
  - DMA/HDMA interactions across CGB memory banks
  - CGB frame output regressions vs known-good snapshots
- ROM tests:
  - mooneye CGB acceptance tests aligned to active subsystem
  - blargg CGB-sensitive timing/sound tests once subsystem support exists
- Command to run: `cargo test --workspace`

## Implementation Steps
1. Phase 12: CGB mode foundation
- Add boot-mode detection and CGB capability flags.
- Implement KEY1 and double-speed switch behavior.
- Add tests for speed-switch sequencing and register semantics.
2. Phase 13: CGB memory model
- Implement VRAM bank switching and WRAM banking.
- Implement HDMA/GDMA behavior and blocking semantics.
- Add banked-memory and DMA correctness tests.
3. Phase 14: CGB PPU features
- Implement tile attributes, palette selection, tile-bank selection, and priority rules.
- Validate scanline/pixel output against targeted CGB visual ROMs.
- Add regression tests around sprite/background priority edge cases.
4. Phase 15: CGB audio/timing stabilization
- Align timing across CPU/PPU/APU in double-speed mode.
- Resolve CGB-specific sound/timing ROM failures.
- Verify SDL/CLI behavior consistency over long-running CGB sessions.
5. Cross-phase ROM governance
- Keep a pinned ROM manifest in docs with each ROM marked `pass`/`fail`/`deferred`.
- Do not advance phases without recorded ROM evidence for the active phase goals.

## Exit Gate
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] CGB ROM manifest updated with phase result deltas

## Notes
- Risks:
  - CGB timing behavior can create cross-subsystem regressions; enforce small, phase-scoped changes.
  - Incomplete ROM manifests lead to ambiguous progress; keep expected outcomes explicit.
- Follow-ups:
  - Create per-phase docs (`phase-12`..`phase-15`) before implementation starts.
