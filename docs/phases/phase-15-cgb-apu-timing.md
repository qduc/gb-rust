# Phase 15: CGB APU Timing & Double-Speed Modeling

## Scope
- Files/modules to change: `crates/gb-core/src/apu/`, `crates/gb-core/src/bus/bus.rs`, `crates/gb-core/src/timer.rs`
- Out of scope: New MBC mappers, PPU rendering refactors.

## Acceptance Criteria
- [x] `roms/gb-test-roms/cgb_sound/rom_singles/03-trigger.gb` passes.
- [x] `roms/gb-test-roms/cgb_sound/rom_singles/09-wave read while on.gb` passes.
- [ ] Double-speed logic verified against hardware-accurate behavior (especially Timer DIV and APU length).

## Tests
- Unit tests: APU frame sequencer clocking quirks.
- Integration tests: `gb-cli` suite runs for `cgb_sound`.
- Command to run: `cargo run -p gb-cli -- suite --rom-dir roms/gb-test-roms/cgb_sound/rom_singles --cycles 400000000`

## Implementation Steps
1. Investigate and fix "Length Counter clocking quirk" in APU (for `03-trigger.gb`).
2. Implement CGB-specific Wave RAM access timing/quirk (for `09-wave read while on.gb`).
3. Audit `Bus::tick` and `Timer::tick` for double-speed accuracy (specifically DIV bit selection).
4. Verify all `cgb_sound` singles pass.

## Exit Gate
- [ ] `cargo fmt --all`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Notes
- `03-trigger.gb` required ordering the "extra length clock" (on NRx4 writes when the sequencer is in the odd phase) before trigger processing, plus CGB's frozen-length re-trigger behavior.
- `09-wave read while on.gb` required wave-channel trigger/readback quirks: the CH3 sample buffer is not refilled on trigger, CGB read-while-on uses a redirected "current byte" view, and the trigger/timer phase is sensitive.
