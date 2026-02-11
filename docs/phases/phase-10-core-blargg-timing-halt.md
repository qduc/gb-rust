# Phase 10: Core Blargg Timing + HALT/Interrupt Fixes

## Status: PARTIAL (core timing fixed, HALT/CGB follow-up pending)

## Scope
- `crates/gb-core/src/cpu/cpu.rs`
- `crates/gb-core/src/cpu/ops.rs`
- `crates/gb-core/src/cpu/cb_ops.rs`
- `crates/gb-core/src/gb.rs`
- `crates/gb-cli/src/main.rs`
- `crates/gb-core/tests/cpu_execution.rs`

## Acceptance Criteria
- `instr_timing.gb` passes
- `mem_timing.gb` and individual `01/02/03` pass
- `halt_bug.gb` completes without timeout and reports deterministic result
- `interrupt_time.gb` completes without timeout and reports deterministic result
- `cpu_instrs` individual ROMs remain passing
- `gb-cli suite` default cycle cap is 100,000,000

## Implementation Steps
- [x] Add per-M-cycle CPU timing integrated into CPU memory access helpers
- [x] Remove external post-step ticking from GameBoy step loop
- [x] Implement HALT bug behavior plumbing (`halt_bug` fetch duplication + halted wake path)
- [x] Add/adjust CPU tests for HALT bug and cycle-driving semantics
- [x] Raise `gb-cli suite` default cycle cap to 100M
- [x] Run fmt, clippy, tests, and target ROM suite

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Results
- 2026-02-11 hardening update:
  - Fixed runtime debug assertion panic in `Cpu::finish_step` by correcting `STOP` (`0x10`) timing to 8 cycles (it consumes the padding byte fetch).
  - Corrected HALT bug trigger semantics: `HALT` now sets `halt_bug` only when `IME=0` and an interrupt is already pending at HALT execution time.
  - Removed incorrect `halt_bug` arming on wake from a previously halted CPU when a new interrupt becomes pending with `IME=0`.
  - Added CPU regression tests for:
    - HALT wake without HALT bug (`halt_wake_on_new_interrupt_does_not_trigger_halt_bug`)
    - STOP cycle accounting (`stop_consumes_padding_byte_and_accounts_full_cycles`)
  - `gb-cli suite --rom-dir gb-test-roms --cycles 20000000` no longer triggers the CPU step-cycle assertion panic under stress.
- Passing after fixes:
  - `instr_timing/instr_timing.gb`
  - `mem_timing/mem_timing.gb`
  - `mem_timing/individual/01-read_timing.gb`
  - `mem_timing/individual/02-write_timing.gb`
  - `mem_timing/individual/03-modify_timing.gb`
- Remaining (deferred):
  - `halt_bug.gb`: still times out without serial output (reproduced at 120M cycles). Targeted trace shows execution reaching `HALT` and then remaining halted at `PC=C07E` (`IME=0`) waiting for an interrupt that never becomes pending, indicating remaining interrupt-generation/timing dependency outside the CPU HALT bug trigger fix.
  - `interrupt_time/interrupt_time.gb`: CGB-only ROM (source declares `.define REQUIRE_CGB 1`). Requires CPU speed switching and CGB-specific features. Not applicable for DMG emulation.
