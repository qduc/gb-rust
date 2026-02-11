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
- Passing after fixes:
  - `instr_timing/instr_timing.gb`
  - `mem_timing/mem_timing.gb`
  - `mem_timing/individual/01-read_timing.gb`
  - `mem_timing/individual/02-write_timing.gb`
  - `mem_timing/individual/03-modify_timing.gb`
  - all `cpu_instrs/individual/01..11`
- Remaining:
  - `halt_bug.gb`: still times out (no serial PASS/FAIL within 200M cycles)
  - `interrupt_time/interrupt_time.gb`: still times out; source declares `.define REQUIRE_CGB 1` (CGB-only ROM)
