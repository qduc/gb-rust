# Game Boy Emulator Implementation Roadmap

Use this checklist in order. Do not move to the next phase until the current one passes its validation commands.

## 1) Baseline ([phase-01-baseline](phases/phase-01-baseline.md))
- [x] Run `cargo check --workspace`
- [x] Run `cargo test --workspace`
- [x] Verify `gb-core`, `gb-cli`, and `gb-sdl` all compile

## 2) Core Architecture Lock ([phase-02-core-architecture-lock](phases/phase-02-core-architecture-lock.md))
- [x] Keep orchestration in `crates/gb-core/src/gb.rs`
- [x] Keep `Cpu::step()` as time driver and `Bus::tick()` as subsystem updater
- [x] Keep frontend concerns out of `gb-core`

## 3) Cartridge + Memory Map ([phase-03-cartridge-memory-map](phases/phase-03-cartridge-memory-map.md))
- [x] Finish `mbc0`
- [x] Implement `mbc1`
- [x] Complete `Bus::read8/write8` mapping
- [x] Add tests for address ranges and bank switching

## 4) CPU Execution Path ([phase-04-cpu-execution-path](phases/phase-04-cpu-execution-path.md))
- [x] Complete opcode decode path
- [x] Implement non-CB ops in `ops.rs`
- [x] Implement CB ops in `cb_ops.rs`
- [x] Add ALU/flag behavior tests
- [x] Add interrupt handling

## 5) Timer, Interrupts, DMA ([phase-05-timer-interrupts-dma](phases/phase-05-timer-interrupts-dma.md))
- [x] Complete timer cycle behavior
- [x] Wire interrupt request/ack flow
- [x] Implement OAM DMA behavior
- [x] Add subsystem integration tests

## 6) PPU (Staged)
Docs: [phase-06-ppu-mode-timing-ly-stat](phases/phase-06-ppu-mode-timing-ly-stat.md), [phase-06-ppu-render-background](phases/phase-06-ppu-render-background.md)
- [x] Implement mode timing + LY/STAT
- [x] Render background
- [x] Render window
- [x] Render sprites
- [x] Expose framebuffer for frontends/tests

## 7) CLI Runner Stabilization
- [x] Add ROM loading and run loop in `gb-cli`
- [x] Add optional trace/debug output
- [x] Use CLI for regression runs

## 8) SDL Frontend Integration
- [x] Present framebuffer in `gb-sdl`
- [x] Wire keyboard/gamepad input
- [x] Integrate audio output incrementally
- [x] Keep emulation behavior in `gb-core`

## 9) Quality Gate (Every Phase)
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`
- [x] Commit one focused subsystem change at a time (see [CONTRIBUTING.md](../CONTRIBUTING.md))

## 10) Final Hardening (DMG/Core)
- [x] Run baseline timing ROMs in CLI (`instr_timing`, `mem_timing`, `cpu_instrs`)
- [x] Resolve remaining DMG ROM-suite failures/timeouts (`halt_bug.gb`)
- [x] Fix runtime panic seen in suite runs (`crates/gb-core/src/cpu/cpu.rs` step cycle assertion)
- [x] Document known limitations in `docs/`

## 11) Scope Decision (Locked)
- [x] Milestone A is **DMG-only release readiness**.
- [x] Milestone B is **CGB expansion**, and is in scope only after Milestone A exit gates pass.
- [x] `halt_bug.gb` is tracked in Milestone A because it is DMG-relevant.
- [x] CGB ROMs (`interrupt_time.gb`, CGB sound/timing behavior) are deferred to Milestone B.

Boundary summary:
- Milestone A (DMG): boot and run DMG titles reliably, pass DMG-oriented CPU/timing/PPU/audio tests, no CGB speed switch or CGB-only hardware behavior required.
- Milestone B (CGB): add double-speed mode, CGB-specific register semantics, VRAM/WRAM banking, CGB PPU extensions, and CGB audio/timing compatibility.

## Current Real-Game Readiness (as of 2026-02-11)
- [x] Ready to run many real DMG games in `gb-sdl`/`gb-cli` for normal gameplay testing.
- [ ] Not yet at full DMG compatibility sign-off.
- Known caveats:
  - `halt_bug.gb` is passed and handled correctly in `gb-cli` via VRAM scraping.
  - `oam_bug` suite is partially fixed. As of 2026-02-11, `2-causes`, `3-non_causes`, `4-scanline_timing`, `5-timing_bug`, and `6-timing_no_bug` pass; `1-lcd_sync` fails and `7-timing_effect` / `8-instr_effect` still do not complete with PASS output.
  - `interrupt_time.gb` is CGB-only and tracked in Milestone B.
  - `dmg_sound` ROMs have mixed results; APU functional but lacking full hardware parity.
  - CGB-only behavior (Double Speed, Banking) foundational work is complete, but expansion is ongoing.

## 12) Milestone A Backlog (Complete Before Any CGB Work)
Execution order is strict:
1. [x] Audio / APU
- [x] Replace APU stub with real DMG APU implementation (channels, frame sequencer, mixing, timing)
- [x] Add APU correctness tests using `dmg_sound` ROM expectations
- [x] Verify stable SDL audio output under long-running gameplay
2. [x] Cartridge / Save Support
- [x] Add missing mapper support needed by real games (MBC5, optionally MBC2)
- [x] Implement MBC3 RTC registers/latching behavior (currently stubbed)
- [x] Add battery-backed SRAM/RTC persistence (`.sav`) load/store
3. [ ] CPU/Timing Stability (High Accuracy)
- [x] Add VRAM-based PASS/FAIL detection for on-screen reporting ROMs (e.g., `halt_bug.gb`)
- [ ] Investigate and fix remaining HALT timing discrepancies and hardware-exact OAM corruption (`oam_bug` suite)
- [ ] Fix remaining sub-instruction timing issues relevant to DMG compatibility
- [x] Keep HALT/timing behavior stable under full suite stress (no debug assertions/panics)
- [x] Re-run full DMG ROM suite with default cap and ensure no regressions
4. [ ] DMA/PPU Accuracy (High Accuracy)
- [x] Model timed OAM DMA and CPU bus restrictions (currently basic implementation)
- [x] Account for 1-MCycle OAM DMA startup delay and specific register blocking windows
- [x] Add tests for edge timing interactions that impact game compatibility (e.g., mid-scanline register writes)
5. [x] Milestone A Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] DMG suite pass set recorded in docs (blargg + selected mooneye DMG cases)

## 13) Milestone B: CGB Expansion Path (In Scope, Starts After Section 12)
Detailed plan doc: [phase-11-cgb-expansion-path](phases/phase-11-cgb-expansion-path.md)

Ordered CGB phases:
- [x] Phase 12: CGB mode foundation (KEY1/speed switch, boot mode detection, CGB-only register gating) — [phase-12-cgb-mode-foundation](phases/phase-12-cgb-mode-foundation.md)
- [ ] Phase 13: CGB memory model (VRAM bank switching, WRAM banking, HDMA/GDMA behavior)
- [ ] Phase 14: CGB PPU features (attributes, palettes, priority rules, tile bank behavior)
- [ ] Phase 15: CGB audio/timing stabilization and frontend validation

CGB ROM strategy (must be tracked per phase):
- [ ] Keep a pinned CGB ROM manifest in docs with expected status per ROM (`pass`/`fail`/`deferred`): [cgb-rom-manifest](cgb-rom-manifest.md)
- [ ] Start with targeted mooneye CGB acceptance tests for the phase feature under development.
- [ ] Add blargg CGB-sensitive ROMs (including CGB sound/timing) once corresponding subsystems land.
- [ ] Require phase-local ROM pass evidence before advancing to the next CGB phase.
