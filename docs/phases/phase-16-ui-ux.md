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
- [x] Add `egui` dependencies.
- [x] Initialize `egui` context.
- [x] In the main loop:
    - [x] Process SDL events -> Pass to `egui`.
    - [x] If `egui` captures input, do not pass to `GameBoy`.
    - [x] Draw `GameBoy` texture.
    - [x] Draw `egui` overlay on top.
    - [x] Present.

### 3. Feature Wiring
- [x] **File Picker**: Use `rfd` to select file -> Drop current `GameBoy`, create new one with selected ROM.
- [x] **State Save/Load**:
    - [x] `File -> Save State`: `bincode::serialize(&gb)` -> write to `rom_name.state`.
    - [x] `File -> Load State`: read file -> `bincode::deserialize` -> replace `gb`.
- [x] **Battery Saves**:
    - [x] On Drop/Exit of `Cartridge` (and periodically), call `save_to_path`.
    - [x] On Load, call `load_from_path`.

### Phase 16 notes
- Implemented menu groups: File, Emulation, Audio, Video, Debug.
- Implemented controls: pause/resume (menu + hotkey), turbo (1x/2x/4x/uncapped), volume scaling, integer scaling toggle, fullscreen toggle.
- Implemented state slots: quick save/load slots 1-3 (menu), plus hotkeys (`F5`/`F8` for slot 1).
- Added ROM open hotkey (`Cmd/Ctrl+O`) and default state hotkeys (`Cmd/Ctrl+S`, `Cmd/Ctrl+L`).

## Exit Gates
- [x] `cargo check` and `cargo test` pass.
- [x] Save States work reliably (load restores exact point).
- [x] Battery saves persist.
- [x] GUI works without crashing SDL.

Current verification status:
- `cargo fmt --all --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace`: PASS (all tests green)
