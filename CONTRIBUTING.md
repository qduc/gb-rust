# Contributing

Thanks for helping improve this repo.

## Keep commits focused by subsystem

This codebase is organized around emulator subsystems (CPU, bus/memory map, cartridge/MBCs, PPU, timer/interrupts/DMA, CLI, SDL frontend). To keep history reviewable and reduce regressions:

- Prefer **one subsystem per commit**.
- Avoid mixing refactors and behavior changes in the same commit.
- If a change spans multiple subsystems, consider splitting into a small series of commits (e.g., "bus: add IO register", then "ppu: use new register").

Commit subjects should stay short, lowercase, and action-oriented (e.g., `timer: fix div increment`).

## Before pushing

Run:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
