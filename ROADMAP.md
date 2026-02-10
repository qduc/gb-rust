# Game Boy Emulator Implementation Roadmap

Use this checklist in order. Do not move to the next phase until the current one passes its validation commands.

## 1) Baseline
- [x] Run `cargo check --workspace`
- [x] Run `cargo test --workspace`
- [x] Verify `gb-core`, `gb-cli`, and `gb-sdl` all compile

## 2) Core Architecture Lock
- [ ] Keep orchestration in `crates/gb-core/src/gb.rs`
- [ ] Keep `Cpu::step()` as time driver and `Bus::tick()` as subsystem updater
- [ ] Keep frontend concerns out of `gb-core`

## 3) Cartridge + Memory Map
- [ ] Finish `mbc0`
- [ ] Implement `mbc1`
- [ ] Complete `Bus::read8/write8` mapping
- [ ] Add tests for address ranges and bank switching

## 4) CPU Execution Path
- [ ] Complete opcode decode path
- [ ] Implement non-CB ops in `ops.rs`
- [ ] Implement CB ops in `cb_ops.rs`
- [ ] Add ALU/flag behavior tests
- [ ] Add interrupt handling

## 5) Timer, Interrupts, DMA
- [ ] Complete timer cycle behavior
- [ ] Wire interrupt request/ack flow
- [ ] Implement OAM DMA behavior
- [ ] Add subsystem integration tests

## 6) PPU (Staged)
- [ ] Implement mode timing + LY/STAT
- [ ] Render background
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
- [ ] Run `cargo fmt --all`
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Run `cargo test --workspace`
- [ ] Commit one focused subsystem change at a time

## 10) Final Hardening
- [ ] Run test ROM suite in CLI
- [ ] Fix timing/determinism regressions
- [ ] Document known limitations in `docs/`
