# Repository Guidelines

## Project Structure & Module Organization
This repository is a Cargo workspace with three crates under `crates/`:
- `crates/gb-core`: Emulator core library (CPU, bus, PPU, APU, cartridge, timer, debug helpers). Supports both **DMG** and **CGB** modes.
- `crates/gb-sdl`: SDL2 desktop frontend with **egui**-based UI overlay.
- `crates/gb-cli`: CLI entrypoint for headless workflows and automated test suites.

Top-level files:
- `Cargo.toml`: workspace members and resolver.
- `docs/project-structure.md`: architecture notes and intended module layout.
- `roms/`: local ROMs for development.

## Documentation Structure
Keep planning and execution notes in `docs/`:
- `docs/project-structure.md`: architecture and module boundaries.
- `docs/roadmap.md`: phase checklist and progress tracking.
- `docs/phases/phase-XX-<name>.md`: one file per implementation phase.

Use `phase-XX` naming to preserve execution order (for example, `phase-16-ui-ux.md`).

## Build, Test, and Development Commands
- `cargo check --workspace`: fast compile checks across all crates.
- `cargo build --workspace`: build everything.
- `cargo test --workspace`: run all tests in the workspace.
- `cargo run -p gb-sdl -- <rom>`: run the SDL frontend.
- `cargo run -p gb-cli -- <rom>`: run the CLI binary.
- `cargo fmt --all`: format all Rust code.
- `cargo clippy --workspace --all-targets -- -D warnings`: lint with warnings treated as errors.

## Coding Style & Naming Conventions
Use standard Rust style:
- 4-space indentation; no tabs.
- `snake_case` for functions/modules/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep modules focused by domain (`cpu/`, `ppu/`, `cartridge/`, etc.), and avoid cross-cutting logic in `main.rs`.
- **Serialization**: Core components (CPU, PPU, etc.) must implement `serde::Serialize` and `serde::Deserialize` to support save states.

## Testing & Tooling
- **gb-cli** features:
  - **Serial Output**: Captures bytes written to SB/SC for automated pass/fail verification.
  - **Cart RAM Output**: Detects Blargg-style results at `$A000` (signature `DE B0 61`), used by `cgb_sound` and others.
  - **VRAM Scraping**: Scrapes the BG tilemap for "Passed"/"Failed" text to detect results for on-screen ROMs (e.g., `halt_bug.gb`).
  - Use `--print-vram` on fail/timeout to see the scraped screen content.
- **gb-sdl** features:
  - **UI Overlay**: Use `Cmd + O` to open ROMs, menu bar for emulation controls.
  - **Save States**: Supports quick-save/load slots and state files (serialized via `bincode`).

## Phase Workflow (Required)
Before implementing any phase, create a short plan document in `docs/phases/` with:
- scope (exact files/modules),
- acceptance criteria,
- tests to add (prefer test-first when behavior is clear),
- implementation steps (3-6 ordered tasks),
- exit gate commands.

Before moving to the next phase, all exit gates must pass:
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Commit & Pull Request Guidelines
Follow the established style:
- Keep subject lines concise and action-oriented (e.g., `cpu: implement halt bug`).
- Prefer **one subsystem per commit** (see `CONTRIBUTING.md`).
- Mix no refactors and behavior changes in the same commit.

For PRs, include:
- what changed and why,
- impacted crates/modules,
- test evidence (commands run, key output).
