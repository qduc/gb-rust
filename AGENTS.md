# Repository Guidelines

## Project Structure & Module Organization
This repository is a Cargo workspace with three crates under `crates/`:
- `crates/gb-core`: emulator core library (CPU, bus, PPU, APU, cartridge, timer, debug helpers).
- `crates/gb-sdl`: SDL2 desktop frontend (`src/main.rs`).
- `crates/gb-cli`: CLI entrypoint for headless workflows (`src/main.rs`).

Top-level files:
- `Cargo.toml`: workspace members and resolver.
- `docs/project-structure.md`: architecture notes and intended module layout.
- `roms/`: local ROMs for development.

## Documentation Structure
Keep planning and execution notes in `docs/`:
- `docs/project-structure.md`: architecture and module boundaries.
- `docs/roadmap.md`: phase checklist and progress tracking (mirror of `ROADMAP.md` when needed).
- `docs/phases/phase-XX-<name>.md`: one file per implementation phase.

Use `phase-XX` naming to preserve execution order (for example, `phase-03-memory-map.md`).

## Build, Test, and Development Commands
- `cargo check --workspace`: fast compile checks across all crates.
- `cargo build --workspace`: build everything.
- `cargo test --workspace`: run all tests in the workspace.
- `cargo run -p gb-sdl`: run the SDL frontend binary.
- `cargo run -p gb-cli`: run the CLI binary.
- `cargo fmt --all`: format all Rust code.
- `cargo clippy --workspace --all-targets -- -D warnings`: lint with warnings treated as errors.

## Coding Style & Naming Conventions
Use standard Rust style:
- 4-space indentation; no tabs.
- `snake_case` for functions/modules/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Keep modules focused by domain (`cpu/`, `ppu/`, `cartridge/`, etc.), and avoid cross-cutting logic in `main.rs`.

Run `cargo fmt --all` before opening a PR. Prefer small, explicit APIs in `gb-core` to keep emulation logic testable.

## Testing Guidelines
Primary test target is `gb-core`.
- Add unit tests next to implementation (`#[cfg(test)] mod tests`) for instruction logic and memory-mapped behavior.
- Add integration tests in `crates/gb-core/tests/` for subsystem interactions.
- Name tests by behavior, for example: `cpu_sets_zero_flag_on_add`.
- Phase gate rule: do not start the next implementation phase until `cargo test --workspace` passes for the current phase.

Execute `cargo test --workspace` locally before pushing.

## Phase Workflow (Required)
Before implementing any phase, create a short plan document in `docs/phases/` with:
- scope (exact files/modules),
- acceptance criteria,
- tests to add (prefer test-first when behavior is clear),
- implementation steps (3-6 ordered tasks),
- exit gate commands.

During implementation:
- update the phase doc as tasks complete,
- keep commits scoped to one subsystem or behavior change.

Before moving to the next phase, all exit gates must pass:
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

## Commit & Pull Request Guidelines
Current history uses short, lowercase commit subjects (for example, `init`, `project structure`). Follow that style:
- Keep subject lines concise and action-oriented.
- Commit related changes together; avoid mixing refactors and behavior changes.

For PRs, include:
- what changed and why,
- impacted crates/modules,
- test evidence (commands run, key output),
- linked issue(s) when applicable.
