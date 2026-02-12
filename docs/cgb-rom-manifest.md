# CGB ROM Manifest

Pinned list of **CGB-related** ROMs currently vendored in this repo (primarily under `roms/`).

Status meanings:
- `pass`: expected to pass in `gb-cli`/`gb-sdl` at HEAD (evidence recorded elsewhere)
- `fail`: expected to fail at HEAD (evidence recorded elsewhere)
- `deferred`: not yet evaluated / no recorded evidence

| ROM (repo path) | Source suite | Expected | Evidence / notes |
|---|---|---:|---|
| `roms/gb-test-roms/interrupt_time/interrupt_time.gb` | blargg `gb-tests` (`interrupt_time`) | fail | `interrupt_time.s` declares `.define REQUIRE_CGB 1`; roadmap notes it currently fails. |
| `roms/gb-test-roms/cgb_sound/cgb_sound.gb` | blargg `gb-tests` (`cgb_sound`) | fail | Still fails at HEAD due to remaining single-ROM failures (`03-trigger`, `09-wave read while on`). Repro: `cargo run -p gb-cli -- suite --rom-dir roms/gb-test-roms/cgb_sound/rom_singles --cycles 400000000` -> `10 passed, 2 failed`. |
| `roms/gb-test-roms/cgb_sound/rom_singles/01-registers.gb` | blargg `gb-tests` (`cgb_sound`) | pass | `cargo run -p gb-cli -- suite --rom-dir roms/gb-test-roms/cgb_sound/rom_singles --cycles 400000000` -> PASS (frames=50 cycles=3714644). |
| `roms/gb-test-roms/cgb_sound/rom_singles/02-len ctr.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=565 cycles=39880008). |
| `roms/gb-test-roms/cgb_sound/rom_singles/03-trigger.gb` | blargg `gb-tests` (`cgb_sound`) | fail | Same run -> FAIL (frames=31 cycles=2400000). With `--print-vram`, cart RAM output: status=0x03, `Enabling in first half of length period should clock length`, `Failed #3`. |
| `roms/gb-test-roms/cgb_sound/rom_singles/04-sweep.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=75 cycles=5470244). |
| `roms/gb-test-roms/cgb_sound/rom_singles/05-sweep details.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=70 cycles=5119124). |
| `roms/gb-test-roms/cgb_sound/rom_singles/06-overflow on trigger.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=55 cycles=4065764). |
| `roms/gb-test-roms/cgb_sound/rom_singles/07-len sweep period sync.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=35 cycles=2661280). |
| `roms/gb-test-roms/cgb_sound/rom_singles/08-len ctr during power.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=145 cycles=10385924). |
| `roms/gb-test-roms/cgb_sound/rom_singles/09-wave read while on.gb` | blargg `gb-tests` (`cgb_sound`) | fail | `cargo run -p gb-cli -- suite --print-vram --cycles 400000000 'roms/gb-test-roms/cgb_sound/rom_singles/09-wave read while on.gb'` -> FAIL (frames=35 cycles=2661280). VRAM shows wave readback byte table ending with an unexpected `74` and CRC-like value `31D750` (still missing a CGB-accurate wave RAM read timing quirk). |
| `roms/gb-test-roms/cgb_sound/rom_singles/10-wave trigger while on.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=235 cycles=16706084). |
| `roms/gb-test-roms/cgb_sound/rom_singles/11-regs after power.gb` | blargg `gb-tests` (`cgb_sound`) | pass | Same run -> PASS (frames=54 cycles=4000000). (Source declares `.define REQUIRE_CGB 1`.) |
| `roms/gb-test-roms/cgb_sound/rom_singles/12-wave.gb` | blargg `gb-tests` (`cgb_sound`) | pass | `cargo run -p gb-cli -- suite --rom-dir roms/gb-test-roms/cgb_sound/rom_singles --cycles 400000000` -> PASS (frames=25 cycles=1959048). |
