## Recommended structure (Cargo workspace)

```
gameboy-emulator/
├─ Cargo.toml                 # workspace
├─ crates/
│  ├─ gb-core/                # pure emulation core (no SDL/windowing)
│  │  ├─ Cargo.toml
│  │  └─ src/
│  │     ├─ lib.rs
│  │     ├─ gb.rs             # GameBoy struct, top-level tick/run_frame
│  │     ├─ cpu/
│  │     │  ├─ mod.rs
│  │     │  ├─ cpu.rs         # registers, step(), interrupt handling
│  │     │  ├─ decode.rs      # opcode decode tables
│  │     │  ├─ ops.rs         # non-CB instruction implementations
│  │     │  └─ cb_ops.rs      # CB-prefixed (0xCBxx) instruction implementations
│  │     ├─ bus/
│  │     │  ├─ mod.rs
│  │     │  └─ bus.rs         # read/write mapping + IO register access
│  │     ├─ cartridge/
│  │     │  ├─ mod.rs
│  │     │  ├─ header.rs
│  │     │  ├─ mbc.rs         # trait + common helpers
│  │     │  ├─ mbc0.rs
│  │     │  ├─ mbc1.rs
│  │     │  └─ mbc3.rs        # optional RTC later
│  │     ├─ ppu/
│  │     │  ├─ mod.rs
│  │     │  ├─ ppu.rs         # modes, timing, registers
│  │     │  ├─ render.rs      # scanline/pixel pipeline
│  │     │  └─ oam.rs
│  │     ├─ apu/
│  │     │  ├─ mod.rs
│  │     │  ├─ apu.rs
│  │     │  └─ channels/      # square, wave, noise
│  │     ├─ timer.rs
│  │     ├─ interrupt.rs
│  │     ├─ input.rs          # joypad register logic
│  │     ├─ dma.rs            # OAM DMA (and later HDMA for GBC)
│  │     ├─ serial.rs         # stub ok initially
│  │     ├─ util/
│  │     │  ├─ mod.rs
│  │     │  └─ bits.rs
│  │     └─ debug/
│  │        ├─ mod.rs
│  │        ├─ trace.rs       # instruction tracing
│  │        └─ views.rs       # tilemap/tile viewer helpers
│  ├─ gb-sdl/                 # desktop frontend (SDL2)
│  │  ├─ Cargo.toml
│  │  └─ src/main.rs
│  └─ gb-cli/                 # optional: headless test runner
│     ├─ Cargo.toml
│     └─ src/main.rs
└─ roms/                      # (optional) local dev folder, gitignored
```

### Why split “core” and “frontend”?

* `gb-core` stays deterministic and testable.
* `gb-sdl` handles window/input/audio without infecting emulation code.
* `gb-cli` is amazing for running test ROMs + traces in CI.

---

## Key types and how they connect

### Top-level: `GameBoy` orchestrator

You want **one owner** of all components to avoid borrow hell.

```rust
pub struct GameBoy {
    pub cpu: Cpu,
    pub bus: Bus,        // owns RAM, PPU, APU, cart, etc. (or references)
}

impl GameBoy {
    pub fn step(&mut self) -> u32 {
        // 1) CPU executes one instruction, returns cycles used
        let cycles = self.cpu.step(&mut self.bus);

        // 2) Tick everything else by the same number of cycles
        self.bus.tick(cycles);

        cycles
    }

    pub fn run_frame(&mut self) {
        // run until PPU reports a completed frame
        while !self.bus.ppu.frame_ready() {
            self.step();
        }
        self.bus.ppu.clear_frame_ready();
    }
}
```

This pattern keeps it simple: CPU “drives” time, everything else follows.

---

## Bus layout (the borrow-checker friendly approach)

**Bus owns the subsystems** and provides `read8/write8`. CPU gets `&mut Bus` during `step()`.

```rust
pub struct Bus {
    pub cart: Cartridge,
    pub ppu: Ppu,
    pub apu: Apu,
    pub timer: Timer,
    pub input: Joypad,
    pub wram: [u8; 0x2000],
    pub hram: [u8; 0x7F],
    pub ie: u8,
    pub iflag: u8,
    // etc.
}

impl Bus {
    pub fn read8(&mut self, addr: u16) -> u8 { /* map address */ }
    pub fn write8(&mut self, addr: u16, val: u8) { /* map address */ }

    pub fn tick(&mut self, cycles: u32) {
        self.timer.tick(cycles, &mut self.iflag);
        self.ppu.tick(cycles, &mut self.iflag);
        self.apu.tick(cycles);
        // DMA/serial etc.
    }
}
```

Why `read8(&mut self, ...)` instead of `&self`?

* Some reads have side effects (certain IO regs, “open bus” behavior later, etc.).
* It avoids needing awkward interior mutability.

---

## Cartridge / MBC design

Use a trait object for MBC logic so you can swap controllers cleanly.

```rust
pub trait Mbc {
    fn read_rom(&self, rom: &[u8], addr: u16) -> u8;
    fn write_rom(&mut self, addr: u16, val: u8);
    fn read_ram(&self, ram: &[u8], addr: u16) -> u8;
    fn write_ram(&mut self, ram: &mut [u8], addr: u16, val: u8);
}

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc: Box<dyn Mbc>,
    pub header: Header,
}
```

Start with `mbc0` (no banking), then add `mbc1`.

---

## CPU module breakdown (so it doesn’t become spaghetti)

* `decode.rs`: tables mapping opcode → function + cycles
* `ops.rs`: non-CB instruction implementations grouped by category
* `cb_ops.rs`: CB-prefixed instruction implementations (bit/rotate/shift/swap family)
* `cpu.rs`: registers, flags, step loop, interrupt handling

That lets you keep `cpu.rs` readable and avoid 2000-line files.

**Expert consensus update:** keep CB-prefixed opcodes in a separate file/module from main opcodes. All three reviews agreed this is the clearest way to keep CPU code maintainable as opcode coverage grows.

---

## PPU module breakdown (same idea)

* `ppu.rs`: mode timing, LY/STAT logic, register reads/writes
* `render.rs`: converting VRAM/OAM to pixels for a scanline/frame
* `oam.rs`: sprite parsing helpers

Also: put the framebuffer in `Ppu` as a `[u32; 160*144]` (or `Vec<u32>`) so the frontend can blit it directly.

---

## Minimal `Cargo.toml` workspace idea

**Top-level `Cargo.toml`:**

```toml
[workspace]
members = ["crates/gb-core", "crates/gb-sdl", "crates/gb-cli"]
resolver = "2"
```

**`gb-core/Cargo.toml`:**

```toml
[package]
name = "gb-core"
version = "0.1.0"
edition = "2021"

[dependencies]
bitflags = "2"
```

**`gb-sdl/Cargo.toml`:**

```toml
[package]
name = "gb-sdl"
version = "0.1.0"
edition = "2021"

[dependencies]
gb-core = { path = "../gb-core" }
sdl2 = "0.36"
```

(Exact versions can vary; this is a normal starting point.)

---

## Where to put tests + test ROM runner

* Put unit tests in `gb-core` (`cpu` flag logic, ALU ops, etc.).
* Add `gb-cli` that runs test ROMs and prints pass/fail + optional trace output.
* Later: snapshot tests for known frames (compare hashes of frame buffer).

---

## Practical tips to keep Rust ergonomic

* Keep ownership simple: `GameBoy` owns `Cpu` and `Bus`, `Bus` owns everything else.
* Avoid `Rc<RefCell<...>>` unless you *really* need it.
* Make “tick” based on cycle counts so accuracy improvements don’t force rewrites.
* Add a `debug` feature flag for tracing/logging so release builds stay fast.
