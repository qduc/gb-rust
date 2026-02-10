# Phase 02: Core Architecture Lock

## Scope
- Files/modules to change:
  - `crates/gb-core/src/gb.rs`
  - `crates/gb-core/src/cpu/cpu.rs`
  - `crates/gb-core/src/bus/bus.rs`
  - `crates/gb-core/src/lib.rs` (only if module boundary adjustments are required)
  - `crates/gb-cli/src/main.rs` and `crates/gb-sdl/src/main.rs` (only if integration wiring requires updates)
- Out of scope:
  - New cartridge, memory-map, CPU opcode, PPU, timer, DMA, or interrupt behavior.
  - Frontend feature work.

## Acceptance Criteria
- [ ] Orchestration remains in `crates/gb-core/src/gb.rs`.
- [ ] `Cpu::step()` remains the time driver and `Bus::tick()` remains the subsystem updater.
- [ ] Frontend concerns remain outside `gb-core`.

## Tests
- Unit tests:
  - Add/adjust minimal tests only if needed to lock orchestration behavior.
- Integration tests:
  - No new integration tests required unless architecture boundaries regress.
- Command to run: `cargo test --workspace`

## Implementation Steps
1. Audit `gb-core` control flow boundaries (`GameBoy::step`, `Cpu::step`, `Bus::tick`).
2. Apply minimal refactors only if orchestration leaks outside `gb.rs`.
3. Validate that frontend-specific concerns are not introduced in `gb-core`.
4. Run workspace quality gates and update roadmap/doc checklists.

## Exit Gate
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Notes
- Risks:
  - Small boundary refactors can unintentionally change execution order.
- Follow-ups:
  - Start Phase 03 only after architecture boundaries and gates are green.
