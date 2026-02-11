# Phase 12: CGB Mode Foundation

## Status: DONE

## Scope
- Files/modules to change:
  - `crates/gb-core/src/cartridge/header.rs`
  - `crates/gb-core/src/cartridge/mod.rs`
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/src/cpu/ops.rs`
  - `crates/gb-core/src/cpu/cpu.rs` (only if needed for STOP/speed-switch sequencing)
  - `crates/gb-core/tests/cartridge_parse.rs`
  - `crates/gb-core/tests/memory_map.rs`
  - `crates/gb-core/tests/cpu_execution.rs`
  - `crates/gb-core/tests/cgb_mode.rs` (new, if test isolation is cleaner)
- Out of scope:
  - VRAM/WRAM bank switching and HDMA/GDMA (Phase 13)
  - CGB PPU tile attributes/palettes/priority rules (Phase 14)
  - CGB audio/timing parity and frontend validation (Phase 15)

## Acceptance Criteria
- [x] Cartridge header parsing exposes CGB capability from header byte `0x0143` (`DMG-only`, `CGB-compatible`, `CGB-only`).
- [x] Emulation mode is selected from cartridge capability (DMG behavior retained for non-CGB ROMs).
- [x] `KEY1` (`0xFF4D`) read/write semantics are implemented for CGB mode:
  - [x] prepare-switch bit write behavior,
  - [x] current-speed bit reporting,
  - [x] correct masked/unused bits behavior.
- [x] `STOP` (`0x10`) performs speed-switch handshake in CGB mode when KEY1 prepare bit is set, and preserves existing DMG STOP behavior otherwise.
- [x] CGB-only register access is gated by mode (DMG reads/writes use documented fallback behavior and do not enable CGB features accidentally).
- [x] All existing DMG tests continue passing without behavior regressions.

## Tests Added
- `cartridge_parse_reports_cgb_capability_from_header_0143`
- `key1_reports_speed_and_prepare_bits_in_cgb_mode`
- `key1_ignores_writes_in_dmg_mode`
- `stop_switches_cpu_speed_only_when_key1_prepare_is_set`
- `stop_keeps_existing_behavior_without_cgb_speed_switch_request`
- `dmg_rom_does_not_expose_cgb_speed_switch_side_effects`

## Implementation Steps
- [x] Add cartridge CGB capability parsing and represent mode intent in `Header`/`Cartridge`.
- [x] Add CGB mode state to core/bus initialization so runtime can branch cleanly on DMG vs CGB behavior.
- [x] Implement `KEY1` register behavior in bus I/O read/write path (`0xFF4D`) with strict bit masking.
- [x] Update `STOP` execution path to run CGB speed-switch handshake when valid, while preserving existing behavior for DMG and non-switch cases.
- [x] Add targeted tests for header parsing, KEY1 semantics, and STOP/speed-switch sequencing.
- [x] Run quality gates and record any deferred edge cases in this doc.

### Deferred / Follow-ups
- Timing scaling for double-speed mode (including any STOP speed-switch settle delay) is intentionally deferred to later phases.
- CGB power-on register defaults (when skipping boot ROM) are not yet modeled.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [ ] Targeted CGB ROM evidence recorded (no mooneye CGB speed-switch ROMs currently vendored under `roms/`)

## Notes
- Risks:
  - STOP semantics are subtle and easy to regress because current implementation maps STOP to halt-like behavior.
  - Incorrect KEY1 masking or mode gating can break ROM feature detection and produce false CGB behavior on DMG paths.
- Follow-ups:
  - Phase 13 consumes this foundation for banking and DMA behavior.
