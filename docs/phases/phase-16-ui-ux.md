# Phase 16: UI/UX & Quality of Life

## Goal
Transform the emulator from a basic runtime into a user-friendly application with a proper GUI, save management, and emulation controls.

## Scope
- **Crates**: `gb-core` (serialization), `gb-sdl` (GUI, input, enhanced main loop).
- **External Deps**:
    - `serde`, `serde_bytes` (for efficient byte array serialization).
    - `bincode` (for compact save state files).
    - `egui`, `egui-sdl2-gl` (or `egui_sdl2_gl` + `gl` if migrating renderer, OR `egui_sdl2_pix` if keeping software interaction). *Note: We will likely need to adjust the rendering pipeline.*
    - `rfd` (native file dialogs).

## Features
1.  **GUI Overlay**
    -   Integrated menu bar (File, Emulation, Audio, Video, Debug).
    -   Settings dialogs for configuration.
2.  **Save System**
    -   **Battery Saves**: Auto-load `.sav` on ROM load, auto-save on exit/interval.
    -   **Save States**: Serialize entire `GameBoy` struct to disk. Quick Save/Load slots.
3.  **Controls**
    -   **Pause/Resume**: Stop `run_frame` loop while GUI is active or via hotkey.
    -   **Turbo**: Uncap FPS or set 2x/4x speed target.
    -   **Volume**: Apply software scaling to audio samples.
4.  **Video**
    -   Integer scaling and Fullscreen toggle.

## Implementation Steps

### 1. Core Serialization (`gb-core`) [COMPLETED]
- [x] Add `serde` with `derive` feature.
- [x] Refactor `Mbc` to `MbcEnum`.
- [x] Derive `Serialize`/`Deserialize` for `GameBoy`, `Cpu`, `Bus`, `Ppu`, `Apu`, `Timer`, `Cartridge`.
- [x] Verify compilation.

### 2. GUI Integration (`gb-sdl`)
-   Add `egui` dependencies.
-   Initialize `egui` context.
-   In the main loop:
    -   Process SDL events -> Pass to `egui`.
    -   If `egui` captures input, do not pass to `GameBoy`.
    -   Draw `GameBoy` texture.
    -   Draw `egui` overlay on top.
    -   Present.

### 3. Feature Wiring
-   **File Picker**: Use `rfd` to select file -> Drop current `GameBoy`, create new one with selected ROM.
-   **State Save/Load**:
    -   `File -> Save State`: `bincode::serialize(&gb)` -> write to `rom_name.state`.
    -   `File -> Load State`: read file -> `bincode::deserialize` -> replace `gb`.
-   **Battery Saves**:
    -   On Drop/Exit of `Cartridge` (or periodically), call `save_to_path`.
    -   On Load, call `load_from_path`.

## Exit Gates
- [ ] `cargo check` and `cargo test` pass.
- [ ] Save States work reliably (load restores exact point).
- [ ] Battery saves persist.
- [ ] GUI works without crashing SDL.
