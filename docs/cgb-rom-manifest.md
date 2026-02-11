# CGB ROM Manifest

Pinned list of **CGB-related** ROMs currently vendored in this repo (primarily under `roms/`).

Status meanings:
- `pass`: expected to pass in `gb-cli`/`gb-sdl` at HEAD (evidence recorded elsewhere)
- `fail`: expected to fail at HEAD (evidence recorded elsewhere)
- `deferred`: not yet evaluated / no recorded evidence

| ROM (repo path) | Source suite | Expected | Evidence / notes |
|---|---|---:|---|
| `roms/gb-test-roms/interrupt_time/interrupt_time.gb` | blargg `gb-tests` (`interrupt_time`) | fail | `interrupt_time.s` declares `.define REQUIRE_CGB 1`; roadmap notes it currently fails. |
| `roms/gb-test-roms/cgb_sound/cgb_sound.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/01-registers.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/02-len ctr.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/03-trigger.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/04-sweep.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/05-sweep details.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/06-overflow on trigger.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/07-len sweep period sync.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/08-len ctr during power.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/09-wave read while on.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/10-wave trigger while on.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | No pass/fail evidence recorded in docs. |
| `roms/gb-test-roms/cgb_sound/rom_singles/11-regs after power.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | Source (`11-regs after power.s`) declares `.define REQUIRE_CGB 1`. |
| `roms/gb-test-roms/cgb_sound/rom_singles/12-wave.gb` | blargg `gb-tests` (`cgb_sound`) | deferred | Source (`12-wave.s`) declares `.define REQUIRE_CGB 1`. |
