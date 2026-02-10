# Phase 01: Baseline

## Scope
- Files/modules to change:
  - Workspace-level validation only (no functional code changes expected).
  - If needed for green builds, minimal compile-fix edits in:
    - `crates/gb-core`
    - `crates/gb-cli`
    - `crates/gb-sdl`
- Out of scope:
  - New emulator behavior/features.
  - Refactors unrelated to baseline compile/test health.

## Acceptance Criteria
- [x] `cargo check --workspace` passes.
- [x] `cargo test --workspace` passes.
- [x] `gb-core`, `gb-cli`, and `gb-sdl` all compile successfully.

## Tests
- Unit tests:
  - Existing crate unit tests must pass without regression.
- Integration tests:
  - Existing workspace integration tests must pass without regression.
- Command to run: `cargo test --workspace`

## Implementation Steps
1. Run `cargo check --workspace` to identify baseline compile issues.
2. Apply minimal, targeted fixes only if compilation fails.
3. Run `cargo test --workspace` and resolve any test regressions.
4. Confirm all three workspace crates compile cleanly.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Notes
- Risks:
  - Baseline failures may reveal cross-crate breakage requiring small compatibility fixes.
- Follow-ups:
  - Start Phase 02 only after all exit gate checks are green.
  - Status: Completed on 2026-02-10.
