# Phase 5: Timer, Interrupts, DMA

## Status: COMPLETE ✓

## Summary
Implemented DMG timer registers + cycle behavior, wired IF semantics for interrupt request/ack flow, and added OAM DMA (FF46) copy behavior. Added 5 new integration tests (timer + DMA) and validated with fmt/clippy/tests.

## Scope
- `crates/gb-core/src/timer.rs`
- `crates/gb-core/src/dma.rs`
- `crates/gb-core/src/bus/bus.rs`
- `crates/gb-core/tests/timer_dma.rs`

## Changes Made

### 1) Timer (`crates/gb-core/src/timer.rs`)
- Implemented internal 16-bit counter with DIV as upper byte
- Falling-edge based TIMA increment model with correct TAC frequency selection
- TIMA overflow reloads TMA and requests Timer interrupt (IF bit 2)
- DIV/TAC writes can trigger a TIMA tick when they create a falling edge

### 2) Bus IO mapping (`crates/gb-core/src/bus/bus.rs`)
- Mapped timer regs: FF04..FF07
- IF (FF0F) now masks writes to 0x1F and reads with 0xE0 high bits set
- FF46 triggers OAM DMA copy

### 3) DMA (`crates/gb-core/src/dma.rs`)
- Implemented `oam_dma()` helper to copy 0xA0 bytes into OAM

### 4) Tests (`crates/gb-core/tests/timer_dma.rs`)
- DIV increment + reset on write
- TIMA frequency stepping
- TIMA overflow reload + interrupt request
- CPU interrupt service path driven by timer overflow
- OAM DMA copy correctness

## Exit Gates (Passing)
- ✓ `cargo fmt --all`
- ✓ `cargo clippy --workspace --all-targets -- -D warnings`
- ✓ `cargo test --workspace`

## Notes
- Known limitation: OAM DMA is currently modeled as an immediate copy on FF46 write (no per-cycle timing/CPU stall yet).
