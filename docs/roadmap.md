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
- [ ] Render background (in progress)
- [ ] Render window
- [ ] Render sprites
- [ ] Expose framebuffer for frontends/tests

## 7) CLI Runner Stabilization
- [ ] Add ROM loading and run loop in `gb-cli`
- [ ] Add optional trace/debug output
- [ ] Use CLI for regression runs

## 8) SDL Frontend Integration
- [ ] Present framebuffer in `gb-sdl`
- [ ] Wire keyboard/gamepad input
- [ ] Integrate audio output incrementally
- [ ] Keep emulation behavior in `gb-core`

## 9) Quality Gate (Every Phase)
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [x] Run `cargo test --workspace`
- [ ] Commit one focused subsystem change at a time

## 10) Final Hardening
- [ ] Run test ROM suite in CLI
- [ ] Fix timing/determinism regressions
- [ ] Document known limitations in `docs/`
