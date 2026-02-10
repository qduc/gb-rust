# Phase 3: Cartridge + Memory Map

## Status: COMPLETE ✓

## Summary
Implemented full cartridge layer (ROM/RAM via MBC) and CPU-visible 16-bit address space with proper routing for all DMG memory regions. All 13 integration tests passing.

## Changes Made

### 1. Header Parsing (`crates/gb-core/src/cartridge/header.rs`)
- `Header` struct with `parse()` method reading offsets 0x0147-0x0149
- `CartridgeType` enum: RomOnly, Mbc1, Mbc1Ram, Mbc1RamBattery, Mbc3, Mbc3Ram, Mbc3RamBattery
- `RomSize` enum with `bank_count()` and `byte_len()` methods (1-128 banks)
- `RamSize` enum with `byte_len()` method (None to 128KB)
- Proper error handling for unsupported/truncated ROMs

### 2. Cartridge (`crates/gb-core/src/cartridge/mod.rs`)
- `Cartridge::from_rom()` constructor with header parsing and MBC selection
- ROM size validation: rejects ROMs smaller than header declares
- Automatic RAM allocation based on header
- MBC factory: routes to Mbc0, Mbc1, or Mbc3

### 3. MBC0 (`crates/gb-core/src/cartridge/mbc0.rs`)
- ROM: direct mapping 0x0000-0x7FFF with bounds checking
- RAM: external RAM at 0xA000-0xBFFF if present, else 0xFF
- No writes to ROM (as per Game Boy)
- Safe indexing via `get()` to prevent panics

### 4. MBC1 (`crates/gb-core/src/cartridge/mbc1.rs`)
- **Register writes:**
  - 0x0000-0x1FFF: RAM enable (pattern 0x0A)
  - 0x2000-0x3FFF: ROM bank low 5 bits (wraps 0 → 1)
  - 0x4000-0x5FFF: Bank high 2 bits (also used for RAM bank select)
  - 0x6000-0x7FFF: Banking mode (0 = 16MB ROM mode, 1 = 4MB ROM + 64KB RAM mode)
- **ROM banking:** Default bank 1 at 0x4000-0x7FFF; bank 0 fixed at 0x0000-0x3FFF
- **RAM banking:** Mode-dependent; proper masking by bank count (not modulo)
- Default state: ROM bank = 1, RAM disabled, mode 0

### 5. MBC3 (`crates/gb-core/src/cartridge/mbc3.rs`)
- Full MBC3 implementation with ROM banking (7-bit bank select)
- RAM enable control
- RAM/RTC select register (RTC stubbed, RAM works)
- No RAM banking (direct 0xA000-0xBFFF mapping)

### 6. Bus Memory Map (`crates/gb-core/src/bus/bus.rs`)
- **ROM:** 0x0000-0x7FFF → MBC read_rom
- **VRAM:** 0x8000-0x9FFF → internal 8KB array
- **Cart RAM:** 0xA000-0xBFFF → MBC read/write_ram
- **WRAM:** 0xC000-0xDFFF → internal 8KB array
- **Echo:** 0xE000-0xFDFF mirrors WRAM (0xC000-0xDFFF)
- **OAM:** 0xFE00-0xFE9F → internal 0xA0-byte array
- **Unusable:** 0xFEA0-0xFEFF → read 0xFF, writes ignored
- **IO:** 0xFF00-0xFF7F → internal register backing (0xFF0F = IF register)
- **HRAM:** 0xFF80-0xFFFE → internal 127-byte array
- **IE:** 0xFFFF → interrupt enable register
- `Bus::new()` constructor for integration with Cartridge

### 7. Tests (`crates/gb-core/tests/memory_map.rs`)
**13 integration tests covering:**
- MBC0 ROM direct mapping, external RAM support
- MBC1 bank switching (ROM low/high bits), mode-dependent RAM banking, RAM enable
- Address ranges: VRAM, OAM, HRAM, IE/IF
- Echo mirroring (0xE000-0xFDFF ↔ 0xC000-0xDFFF)
- Unusable region behavior

All tests passing.

## Key Design Decisions

1. **ROM size validation:** Reject undersized ROMs early to catch cartridge issues
2. **Masking over modulo:** RAM banking masks bank index by `(ram.len()/0x2000)` instead of modulo address, correctly handling partial banks
3. **Wrapping subtraction:** Use `addr.wrapping_sub(0xA000)` for robustness across all bus addresses
4. **Safe indexing:** All ROM/RAM reads use `get()` → 0xFF fallback, preventing panics
5. **Default MBC1 state:** rom_bank_low5 = 1 (matches Game Boy behavior of bank 1 visible at 0x4000 on boot)

## Exit Gates (Passing)
- ✓ `cargo fmt --all`
- ✓ `cargo clippy --workspace --all-targets -- -D warnings`
- ✓ `cargo test --workspace` (13 integration tests + unit tests)

## Known Limitations
- RTC (real-time clock) in MBC3 is stubbed (always reads/writes 0)
- Cartridge battery/SRAM persistence not implemented (expected for future phase)
- No support for MBC2, MBC5, etc. (can be added as needed)

## Next Phase
Phase 4: CPU Execution Path (opcode decode, ALU, flags, interrupt handling)
