# gb-rust

A modular Game Boy emulator written in Rust.

`gb-rust` is designed with a clear separation between its emulation core and its frontends, making it both highly testable and adaptable.

## Project Structure

This repository is organized as a Cargo workspace with three primary crates:

- **[gb-core](crates/gb-core)**: The pure emulation engine. It contains the logic for the CPU, PPU, APU, Bus, Cartridge mappers (MBC), and other hardware components. It has no dependencies on windowing or audio APIs.
- **[gb-sdl](crates/gb-sdl)**: A desktop frontend using SDL2 for rendering, input, and audio performance.
- **[gb-cli](crates/gb-cli)**: A headless CLI tool for running ROMs, performing instruction traces, and running automated test suites.

## Features

- **CPU**: Full implementation of the Game Boy instruction set, including correct timing and interrupt handling.
- **PPU**: Support for background, window, and sprite layers.
- **APU**: DMG-style audio support (Square 1, Square 2, Wave, and Noise channels).
- **Cartridge Support**:
  - MBC0 (No mapper)
  - MBC1
  - MBC3 (with RTC and Battery support)
  - MBC5
- **UI & UX**:
  - **egui Overlay**: In-app menu bar for configuration and controls.
  - **Save States**: Quick save/load slots and state file support.
  - **Input**: Support for keyboard and gamepad via SDL2, with remapping support.
- **Persistence**: Battery-backed SRAM and RTC state are saved to/loaded from `.sav` files.
- **Testing**: Integrated test runner in `gb-cli` with serial output, cart RAM detection, and VRAM tilemap scraping.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable version)
- [SDL2](https://www.libsdl.org/) development libraries (for `gb-sdl`)

### Building

To build the entire workspace:

```bash
cargo build --release
```

### Running

To run a ROM using the SDL frontend:

```bash
cargo run -p gb-sdl -- path/to/rom.gb
```

To run a ROM in the headless CLI (useful for testing):

```bash
cargo run -p gb-cli -- path/to/rom.gb
```

## Development

### Running Tests

```bash
cargo test --workspace
```

### Linting and Formatting

We follow standard Rust style guidelines:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

## Roadmap

The project has completed **Milestone A (DMG)** and **Milestone B (CGB Expansion)**. We are currently working on **Milestone C (UI/UX & Quality of Life)**.

See [docs/roadmap.md](docs/roadmap.md) for detailed progress and [docs/project-structure.md](docs/project-structure.md) for architectural details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details (if available).
