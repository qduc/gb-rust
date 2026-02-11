# Phase 11: APU DMG Baseline

## Status: COMPLETE (DMG baseline)

## Scope
- `crates/gb-core/src/apu/apu.rs`
- `crates/gb-core/src/apu/channels/mod.rs`
- `crates/gb-core/src/apu/channels/square.rs`
- `crates/gb-core/src/apu/channels/wave.rs`
- `crates/gb-core/src/apu/channels/noise.rs`
- `crates/gb-core/src/bus/bus.rs`
- `crates/gb-core/tests/apu.rs`
- `crates/gb-sdl/src/audio.rs` (only if required)

## Acceptance Criteria
- Replace APU stub with DMG-oriented baseline behavior:
  - channel state for CH1/CH2/CH3/CH4,
  - frame sequencer timing (length/sweep/envelope cadence),
  - register read/write behavior for `NR10..NR52` and wave RAM,
  - practical stereo mix output with sample buffering.
- Bus routes APU register addresses through `Apu` register accessors.
- Add APU tests aligned with observable `dmg_sound` expectations where practical.
- SDL audio pumping remains stable with produced APU samples and does not regress queue handling behavior.

## Tests To Add
- `apu_register_masks_match_dmg_expectations`
- `apu_power_off_clears_regs_but_preserves_wave_ram`
- `apu_ignores_register_writes_while_powered_off`
- `apu_trigger_and_length_counter_drive_nr52_status`
- `apu_emits_interleaved_stereo_samples`
- `apu_long_run_sample_rate_stays_stable`

## Implementation Steps
- [x] Add phase doc and track progress while implementing.
- [x] Implement channel modules (square/wave/noise) with trigger, timer, length, and output logic.
- [x] Implement APU core timing (frame sequencer + sample clock), register semantics, power behavior, and mixing.
- [x] Wire `Bus` I/O map for `0xFF10..=0xFF3F` reads/writes to APU.
- [x] Add/iterate APU integration tests.
- [x] Run quality gates and record results.

## Exit Gate
- [x] `cargo fmt --all`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Results
- Implemented:
  - DMG-oriented APU baseline with four channels, frame sequencer, timing clocking, register model, power behavior, wave RAM persistence, and stereo sample generation.
  - Bus register routing for `0xFF10..=0xFF3F` to APU read/write handlers.
  - APU integration tests aligned to observable `dmg_sound` register/power expectations plus sample/status sanity checks.
- Deferred:
  - Full `dmg_sound` ROM-suite behavior parity for edge-trigger quirks (length enable timing edge cases, exact sweep corner cases, wave-on read/write quirks under all timing races).
  - High-fidelity analog filtering and resampling characteristics of original DMG hardware.
  - No SDL audio code changes were required; existing queue/backpressure logic remains unchanged.
