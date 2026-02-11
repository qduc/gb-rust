# Phase 10 (Milestone A): OAM Corruption Accuracy

## Status: IN PROGRESS

## Scope
- Files/modules to change:
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/src/cpu/cpu.rs`
  - `crates/gb-core/src/cpu/ops.rs`
  - `crates/gb-core/src/ppu/ppu.rs` (timing alignment only if required)
  - `docs/roadmap.md`
- Out of scope:
  - CGB-specific timing and memory behavior
  - Non-DMG frontend feature work

## Acceptance Criteria
- [ ] `oam_bug` multi-ROM reports full pass.
- [x] OAM bug triggers are modeled for CPU reads/writes to `0xFE00..=0xFEFF` during mode 2.
- [x] OAM bug triggers are modeled for IDU-backed instructions (`INC/DEC rr`, `LD A,(HL+/-)`, `PUSH/POP`).
- [ ] Corruption pattern matches timing-dependent and instruction-effect ROMs (`7-timing_effect`, `8-instr_effect`).
- [ ] LCD-enable sync behavior matches `1-lcd_sync`.

## Tests
- [x] `cargo test --workspace`
- [x] ROM evidence run for `oam_bug` singles.

Current ROM evidence (2026-02-11):
- PASS: `2-causes`, `3-non_causes`, `4-scanline_timing`, `5-timing_bug`, `6-timing_no_bug`
- FAIL/TIMEOUT: `1-lcd_sync`, `7-timing_effect`, `8-instr_effect`

## Implementation Steps
1. [x] Add mode-2 OAM corruption triggers in bus access paths.
2. [x] Add IDU-driven corruption hooks in CPU instruction paths.
3. [x] Validate timing-window ROMs and non-cause ROMs.
4. [ ] Close remaining LCD sync + instruction/timing pattern mismatches.
5. [ ] Re-run full `oam_bug` and update roadmap caveats.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [ ] `oam_bug` full pass evidence recorded

## Next Session Checklist
1. Reproduce baseline before changes: run `oam_bug/oam_bug.gb` and confirm current status (`01:03`, `02..06:ok`, unresolved `07`).
2. Fix LCD enable scanline start alignment to satisfy `rom_singles/1-lcd_sync.gb` without regressing `4/5/6`.
3. Finish `POP`/`LD A,(HL+/-)` IDU corruption sequencing to satisfy `rom_singles/8-instr_effect.gb`.
4. Close timing-dependent row corruption mismatch in `rom_singles/7-timing_effect.gb`.
5. Re-run all singles and multi-ROM with `--print-vram`, then update `docs/roadmap.md` with exact pass/fail counts.

## Related But Not Modified In This Step
- `crates/gb-core/src/dma.rs` (OAM DMA timing model influences OAM-visible state during tests)
- `crates/gb-core/tests/timer_dma.rs` (DMA/PPU interaction regressions should be checked while iterating OAM bug fixes)
- `crates/gb-cli/src/main.rs` (VRAM scraping output is the primary oracle for `oam_bug` progress)
