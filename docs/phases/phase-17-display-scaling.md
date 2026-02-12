# Phase 17: Display Scaling Improvements

## Scope
- `crates/gb-sdl/src/main.rs`: Implement fixed scaling options and window auto-resizing.

## Acceptance Criteria
- New scaling options in the Video menu: Fit, 1x, 1.5x, 2x, 3x, 4x, 5x.
- Selecting a fixed scaling option (1x-5x) automatically resizes the SDL window.
- The window size should account for the menu bar and status bar so the game area matches the selected scale as closely as possible.
- "Integer scaling" still applies to "Fit" mode.

## Implementation Steps
1. Define `DisplayScale` enum with supported scale factors.
2. Add `display_scale` to `App` struct.
3. Update `App::ui` to show scaling options in the "Video" menu.
4. Add a mechanism to request a window resize from within `App::ui`.
5. Implement the resize logic in the main loop, using `window.set_size()`.
6. Adjust the central panel logic to respect the fixed scale if not in "Fit" mode.

## Exit Gate Commands
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
