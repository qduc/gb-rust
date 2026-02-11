use std::io::Write;
use std::path::{Path, PathBuf};

use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;
use gb_core::gb::GameBoy;

#[derive(Debug)]
enum Command {
    Run(RunArgs),
    Suite(SuiteArgs),
    SelfTest(SelfTestArgs),
}

#[derive(Debug)]
struct RunArgs {
    rom_path: PathBuf,
    max_frames: Option<u64>,
    max_cycles: Option<u64>,
    headless: bool,
    verbose: bool,
    trace_cpu: bool,
    trace_ppu: bool,
    log_serial: bool,
    print_serial: bool,
    print_vram: bool,
}

#[derive(Debug)]
struct SuiteArgs {
    rom_dir: PathBuf,
    rom_paths: Vec<PathBuf>,
    max_frames: Option<u64>,
    max_cycles: Option<u64>,
    pass_text: Vec<String>,
    fail_text: Vec<String>,
    print_serial: bool,
    print_vram: bool,
}

#[derive(Debug)]
struct SelfTestArgs {
    max_cycles: Option<u64>,
    pass_text: Vec<String>,
    fail_text: Vec<String>,
    print_serial: bool,
    print_vram: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum RomResult {
    Pass,
    Fail,
    Timeout,
}

impl RomResult {
    fn as_str(self) -> &'static str {
        match self {
            RomResult::Pass => "PASS",
            RomResult::Fail => "FAIL",
            RomResult::Timeout => "TIMEOUT",
        }
    }
}

fn print_usage() {
    eprintln!(
        "Usage:\n\
  gb-cli <rom.gb> [--frames N] [--cycles N] [--headless] [-v|--verbose]\n\
        [--trace-cpu] [--trace-ppu] [--log-serial] [--print-serial]\n\
  gb-cli run <rom.gb> [--frames N] [--cycles N] [--headless] [-v|--verbose]\n\
        [--trace-cpu] [--trace-ppu] [--log-serial] [--print-serial]\n\
  gb-cli suite [--rom-dir DIR] [--frames N] [--cycles N] [--pass-text S] [--fail-text S] [--print-serial] [ROM...]+\n\
  gb-cli self-test [--cycles N] [--pass-text S] [--fail-text S] [--print-serial]\n\
\n\
Commands:\n\
  run        Run a single ROM (default if no subcommand is given).\n\
  suite      Discover and run a set of ROMs (default dir: ./roms).\n\
  self-test  Run a tiny built-in ROM that prints 'Passed' via serial.\n\
\n\
Optional debug output (run command):\n\
  -v, --verbose   Print ROM metadata + run summary (stderr).\n\
  --trace-cpu     Print per-instruction CPU trace (stderr).\n\
  --trace-ppu     Print PPU LY/mode transitions (stderr).\n\
  --log-serial    Stream serial output to stdout as it is produced.\n\
  --print-serial  Print captured serial output at the end.\n\
\n\
Suite pass/fail detection:\n\
  - Captures bytes written to SB (0xFF01) when SC (0xFF02) is written with bit7 set\n\
    (common in blargg/mooneye test ROMs).\n\
  - Marks PASS if output contains any --pass-text (default: 'passed').\n\
  - Marks FAIL if output contains any --fail-text (default: 'failed', 'fail').\n\
  - Otherwise stops at limits and marks TIMEOUT.\n"
    );
    eprintln!("  --print-vram    Print scraped BG tilemap text on FAIL/TIMEOUT.");
}

fn parse_args() -> Result<Command, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err("missing args".to_string());
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            print_usage();
            std::process::exit(0);
        }
        "run" => parse_run_args(&args[1..]).map(Command::Run),
        "suite" => parse_suite_args(&args[1..]).map(Command::Suite),
        "self-test" => parse_self_test_args(&args[1..]).map(Command::SelfTest),
        _ => parse_run_args(&args).map(Command::Run),
    }
}

fn parse_run_args(args: &[String]) -> Result<RunArgs, String> {
    if args.is_empty() {
        return Err("missing ROM path".to_string());
    }

    let mut it = args.iter();
    let rom_path = PathBuf::from(it.next().unwrap());

    let mut max_frames: Option<u64> = None;
    let mut max_cycles: Option<u64> = None;
    let mut headless = false;
    let mut verbose = false;
    let mut trace_cpu = false;
    let mut trace_ppu = false;
    let mut log_serial = false;
    let mut print_serial = false;
    let mut print_vram = false;

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--headless" => headless = true,
            "-v" | "--verbose" => verbose = true,
            "--trace-cpu" => trace_cpu = true,
            "--trace-ppu" => trace_ppu = true,
            "--log-serial" => log_serial = true,
            "--print-serial" => print_serial = true,
            "--print-vram" => print_vram = true,
            "--frames" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--frames requires a value".to_string())?;
                max_frames = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("invalid --frames value: {v}"))?,
                );
            }
            "--cycles" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--cycles requires a value".to_string())?;
                max_cycles = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("invalid --cycles value: {v}"))?,
                );
            }
            _ if arg.starts_with('-') => return Err(format!("unknown flag: {arg}")),
            _ => return Err(format!("unexpected extra positional arg: {arg}")),
        }
    }

    Ok(RunArgs {
        rom_path,
        max_frames,
        max_cycles,
        headless,
        verbose,
        trace_cpu,
        trace_ppu,
        log_serial,
        print_serial,
        print_vram,
    })
}

fn parse_suite_args(args: &[String]) -> Result<SuiteArgs, String> {
    let mut rom_dir = PathBuf::from("roms");
    let mut rom_paths: Vec<PathBuf> = Vec::new();
    let mut max_frames: Option<u64> = None;
    let mut max_cycles: Option<u64> = Some(300_000_000);
    let mut pass_text = vec!["passed".to_string()];
    let mut fail_text = vec!["failed".to_string(), "fail".to_string()];
    let mut print_serial = false;
    let mut print_vram = false;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--rom-dir" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--rom-dir requires a value".to_string())?;
                rom_dir = PathBuf::from(v);
            }
            "--frames" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--frames requires a value".to_string())?;
                max_frames = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("invalid --frames value: {v}"))?,
                );
            }
            "--cycles" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--cycles requires a value".to_string())?;
                max_cycles = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("invalid --cycles value: {v}"))?,
                );
            }
            "--pass-text" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--pass-text requires a value".to_string())?;
                pass_text.push(v.to_string());
            }
            "--fail-text" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--fail-text requires a value".to_string())?;
                fail_text.push(v.to_string());
            }
            "--print-serial" => print_serial = true,
            "--print-vram" => print_vram = true,
            _ if arg.starts_with('-') => return Err(format!("unknown flag: {arg}")),
            _ => rom_paths.push(PathBuf::from(arg)),
        }
    }

    Ok(SuiteArgs {
        rom_dir,
        rom_paths,
        max_frames,
        max_cycles,
        pass_text,
        fail_text,
        print_serial,
        print_vram,
    })
}

fn parse_self_test_args(args: &[String]) -> Result<SelfTestArgs, String> {
    let mut max_cycles: Option<u64> = Some(5_000_000);
    let mut pass_text = vec!["passed".to_string()];
    let mut fail_text = vec!["failed".to_string(), "fail".to_string()];
    let mut print_serial = false;
    let mut print_vram = false;

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--cycles" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--cycles requires a value".to_string())?;
                max_cycles = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("invalid --cycles value: {v}"))?,
                );
            }
            "--pass-text" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--pass-text requires a value".to_string())?;
                pass_text.push(v.to_string());
            }
            "--fail-text" => {
                let v = it
                    .next()
                    .ok_or_else(|| "--fail-text requires a value".to_string())?;
                fail_text.push(v.to_string());
            }
            "--print-serial" => print_serial = true,
            "--print-vram" => print_vram = true,
            _ if arg.starts_with('-') => return Err(format!("unknown flag: {arg}")),
            _ => return Err(format!("unexpected positional arg: {arg}")),
        }
    }

    Ok(SelfTestArgs {
        max_cycles,
        pass_text,
        fail_text,
        print_serial,
        print_vram,
    })
}

fn init_dmg_post_boot(gb: &mut GameBoy) {
    // DMG (no-boot-rom) register values commonly used by emulators.
    gb.cpu.a = 0x01;
    gb.cpu.f = 0xB0;
    gb.cpu.b = 0x00;
    gb.cpu.c = 0x13;
    gb.cpu.d = 0x00;
    gb.cpu.e = 0xD8;
    gb.cpu.h = 0x01;
    gb.cpu.l = 0x4D;
    gb.cpu.sp = 0xFFFE;
    gb.cpu.pc = 0x0100;

    gb.bus.ie = 0x00;
    gb.bus.iflag = 0x00;

    // Initialize key IO registers (enough for typical test ROMs).
    // Use bus writes to respect any masking side effects.
    let io_inits: &[(u16, u8)] = &[
        (0xFF00, 0xCF),
        (0xFF05, 0x00),
        (0xFF06, 0x00),
        (0xFF07, 0x00),
        (0xFF10, 0x80),
        (0xFF11, 0xBF),
        (0xFF12, 0xF3),
        (0xFF14, 0xBF),
        (0xFF16, 0x3F),
        (0xFF17, 0x00),
        (0xFF19, 0xBF),
        (0xFF1A, 0x7F),
        (0xFF1B, 0xFF),
        (0xFF1C, 0x9F),
        (0xFF1E, 0xBF),
        (0xFF20, 0xFF),
        (0xFF21, 0x00),
        (0xFF22, 0x00),
        (0xFF23, 0xBF),
        (0xFF24, 0x77),
        (0xFF25, 0xF3),
        (0xFF26, 0xF1),
        (0xFF40, 0x91),
        (0xFF42, 0x00),
        (0xFF43, 0x00),
        (0xFF45, 0x00),
        (0xFF47, 0xFC),
        (0xFF48, 0xFF),
        (0xFF49, 0xFF),
        (0xFF4A, 0x00),
        (0xFF4B, 0x00),
    ];

    for &(addr, val) in io_inits {
        gb.bus.write8(addr, val);
    }
}

fn discover_roms(dir: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(out: &mut Vec<PathBuf>, p: &Path) -> Result<(), String> {
        let rd = std::fs::read_dir(p)
            .map_err(|e| format!("failed to read ROM directory {}: {e}", p.display()))?;
        for ent in rd {
            let ent = ent.map_err(|e| format!("failed to read entry in {}: {e}", p.display()))?;
            let path = ent.path();
            if path.is_dir() {
                visit(out, &path)?;
                continue;
            }
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if matches!(ext.to_ascii_lowercase().as_str(), "gb" | "gbc") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    visit(&mut out, dir)?;
    out.sort();
    Ok(out)
}

fn contains_any(haystack_lower: &str, needles: &[String]) -> bool {
    needles
        .iter()
        .any(|n| !n.is_empty() && haystack_lower.contains(&n.to_ascii_lowercase()))
}

fn decode_blargg_screen_char(tile_id: u8) -> u8 {
    // Some GB test ROMs display ASCII directly by putting character codes in the BG tilemap.
    // Many also set the high bit; masking with 0x7F matches common conventions.
    let c = tile_id & 0x7F;
    if (0x20..=0x7E).contains(&c) {
        c
    } else {
        b' '
    }
}

fn scrape_bg_tilemap_text_lower(vram: &[u8], map_offset: usize) -> String {
    // BG tilemap is 32x32 bytes. In DMG VRAM, maps live at:
    // - 0x9800..=0x9BFF -> offset 0x1800
    // - 0x9C00..=0x9FFF -> offset 0x1C00
    const MAP_W: usize = 32;
    const MAP_H: usize = 32;
    const MAP_SIZE: usize = MAP_W * MAP_H;

    let mut out: Vec<u8> = Vec::with_capacity(MAP_SIZE + MAP_H);
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let i = y * MAP_W + x;
            let tile_id = vram[map_offset + i];
            out.push(decode_blargg_screen_char(tile_id));
        }
        out.push(b'\n');
    }
    String::from_utf8_lossy(&out).to_ascii_lowercase()
}

fn scrape_all_bg_text_lower(bus: &Bus) -> String {
    // Try both BG tilemaps. Some ROMs use LCDC bit3 to pick one; others might write either.
    let t9800 = scrape_bg_tilemap_text_lower(&bus.vram, 0x1800);
    let t9c00 = scrape_bg_tilemap_text_lower(&bus.vram, 0x1C00);
    // Keep it simple: concatenate so substring search can hit either.
    // (Separators are spaces/newlines to avoid accidentally concatenating words.)
    format!("{t9800}\n{t9c00}")
}

fn scrape_bg_tilemap_text(vram: &[u8], map_offset: usize) -> String {
    const MAP_W: usize = 32;
    const MAP_H: usize = 32;
    let mut out: Vec<u8> = Vec::with_capacity(MAP_W * MAP_H + MAP_H);
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let i = y * MAP_W + x;
            let tile_id = vram[map_offset + i];
            out.push(decode_blargg_screen_char(tile_id));
        }
        out.push(b'\n');
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn scrape_all_bg_text(bus: &Bus) -> String {
    let t9800 = scrape_bg_tilemap_text(&bus.vram, 0x1800);
    let t9c00 = scrape_bg_tilemap_text(&bus.vram, 0x1C00);
    format!("{t9800}\n{t9c00}")
}

fn run_for_serial_result(
    cart: Cartridge,
    max_frames: Option<u64>,
    max_cycles: Option<u64>,
    pass_text: &[String],
    fail_text: &[String],
    print_vram: bool,
) -> (RomResult, Vec<u8>, u64, u64) {
    let mut gb = GameBoy {
        cpu: Cpu::new(),
        bus: Bus::new(cart),
    };
    init_dmg_post_boot(&mut gb);

    let mut frames: u64 = 0;
    let mut cycles: u64 = 0;
    let mut output: Vec<u8> = Vec::new();

    loop {
        if max_frames.is_some_and(|m| frames >= m) || max_cycles.is_some_and(|m| cycles >= m) {
            // Last-chance VRAM scrape: some ROMs (e.g. blargg halt_bug.gb) report results on-screen.
            let screen_lower = scrape_all_bg_text_lower(&gb.bus);
            if contains_any(&screen_lower, fail_text) {
                if print_vram {
                    println!(
                        "--- VRAM BG tilemap (on FAIL) ---\n{}",
                        scrape_all_bg_text(&gb.bus)
                    );
                }
                return (RomResult::Fail, output, frames, cycles);
            }
            if contains_any(&screen_lower, pass_text) {
                return (RomResult::Pass, output, frames, cycles);
            }
            if print_vram {
                println!(
                    "--- VRAM BG tilemap (on TIMEOUT) ---\n{}",
                    scrape_all_bg_text(&gb.bus)
                );
            }
            return (RomResult::Timeout, output, frames, cycles);
        }

        cycles += gb.step() as u64;

        let new = gb.bus.serial.take_output();
        if !new.is_empty() {
            output.extend_from_slice(&new);
            let out_lower = String::from_utf8_lossy(&output).to_ascii_lowercase();
            if contains_any(&out_lower, fail_text) {
                if print_vram {
                    println!(
                        "--- VRAM BG tilemap (on FAIL) ---\n{}",
                        scrape_all_bg_text(&gb.bus)
                    );
                }
                return (RomResult::Fail, output, frames, cycles);
            }
            if contains_any(&out_lower, pass_text) {
                return (RomResult::Pass, output, frames, cycles);
            }
        }

        if gb.bus.ppu.frame_ready() {
            frames += 1;
            gb.bus.ppu.clear_frame_ready();

            // VRAM fallback: check for on-screen "Passed"/"Failed" text.
            // Keep it cheap-ish: check early frames and then every few frames.
            if frames <= 3 || frames.is_multiple_of(5) {
                let screen_lower = scrape_all_bg_text_lower(&gb.bus);
                if contains_any(&screen_lower, fail_text) {
                    if print_vram {
                        println!(
                            "--- VRAM BG tilemap (on FAIL) ---\n{}",
                            scrape_all_bg_text(&gb.bus)
                        );
                    }
                    return (RomResult::Fail, output, frames, cycles);
                }
                if contains_any(&screen_lower, pass_text) {
                    return (RomResult::Pass, output, frames, cycles);
                }
            }
        }
    }
}

fn make_self_test_rom() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];

    // Jump over the cartridge header area (0x0100..=0x014F).
    let start = 0x0150usize;
    rom[0x0100] = 0xC3; // JP a16
    rom[0x0101] = (start & 0xFF) as u8;
    rom[0x0102] = (start >> 8) as u8;

    let mut pc = start;
    for &b in b"Passed\n" {
        // LD A, d8
        rom[pc] = 0x3E;
        rom[pc + 1] = b;
        pc += 2;
        // LD (a16), A  ; SB (FF01)
        rom[pc] = 0xEA;
        rom[pc + 1] = 0x01;
        rom[pc + 2] = 0xFF;
        pc += 3;
        // LD A, d8 (0x81)
        rom[pc] = 0x3E;
        rom[pc + 1] = 0x81;
        pc += 2;
        // LD (a16), A  ; SC (FF02)
        rom[pc] = 0xEA;
        rom[pc + 1] = 0x02;
        rom[pc + 2] = 0xFF;
        pc += 3;
    }
    // JR -2 (infinite loop)
    rom[pc] = 0x18;
    rom[pc + 1] = 0xFE;

    // Minimal header bytes needed by Cartridge::from_rom.
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KiB
    rom[0x0149] = 0x00; // no RAM

    rom
}

fn run_single(args: RunArgs) -> Result<i32, String> {
    let rom = std::fs::read(&args.rom_path)
        .map_err(|e| format!("failed to read ROM {}: {e}", args.rom_path.display()))?;
    let cart = Cartridge::from_rom(rom).map_err(|e| format!("invalid ROM: {e:?}"))?;

    if args.verbose {
        eprintln!(
            "Loaded ROM: {} ({:?}, {:?}, {:?})",
            args.rom_path.display(),
            cart.header.cartridge_type,
            cart.header.rom_size,
            cart.header.ram_size
        );
    }

    let mut gb = GameBoy {
        cpu: Cpu::new(),
        bus: Bus::new(cart),
    };
    init_dmg_post_boot(&mut gb);

    let mut frames: u64 = 0;
    let mut cycles: u64 = 0;

    let mut last_ly: u8 = gb.bus.io[0x44];
    let mut last_mode: u8 = gb.bus.io[0x41] & 0x03;

    let mut serial_out: Vec<u8> = Vec::new();
    let mut serial_batch: Vec<u8> = Vec::new();
    let mut stdout = std::io::stdout();

    loop {
        if args.max_frames.is_some_and(|m| frames >= m)
            || args.max_cycles.is_some_and(|m| cycles >= m)
        {
            if args.print_vram {
                println!(
                    "--- VRAM BG tilemap (on TIMEOUT) ---\n{}",
                    scrape_all_bg_text(&gb.bus)
                );
            }
            break;
        }

        if args.trace_cpu {
            let pc = gb.cpu.pc;
            let b0 = gb.bus.read8(pc);
            let b1 = gb.bus.read8(pc.wrapping_add(1));
            let b2 = gb.bus.read8(pc.wrapping_add(2));
            eprintln!(
                "CYC={cycles:010} PC={pc:04X} OP={b0:02X} {b1:02X} {b2:02X} AF={:02X}{:02X} BC={:02X}{:02X} DE={:02X}{:02X} HL={:02X}{:02X} SP={:04X} IME={} HALT={} IE={:02X} IF={:02X}",
                gb.cpu.a,
                gb.cpu.f,
                gb.cpu.b,
                gb.cpu.c,
                gb.cpu.d,
                gb.cpu.e,
                gb.cpu.h,
                gb.cpu.l,
                gb.cpu.sp,
                gb.cpu.ime,
                gb.cpu.halted,
                gb.bus.ie,
                gb.bus.iflag
            );
            let step_cycles = gb.cpu.step(&mut gb.bus);
            gb.bus.tick(step_cycles);
            cycles += step_cycles as u64;
        } else {
            cycles += gb.step() as u64;
        }

        if args.trace_ppu {
            let ly = gb.bus.io[0x44];
            let mode = gb.bus.io[0x41] & 0x03;
            if ly != last_ly || mode != last_mode {
                eprintln!("PPU ly={ly} mode={mode}");
                last_ly = ly;
                last_mode = mode;
            }
        }

        serial_batch.extend(gb.bus.serial.drain_output());
        if !serial_batch.is_empty() {
            if args.log_serial {
                stdout
                    .write_all(&serial_batch)
                    .map_err(|e| format!("failed to write serial output: {e}"))?;
                stdout
                    .flush()
                    .map_err(|e| format!("failed to flush serial output: {e}"))?;
            }
            if args.print_serial {
                serial_out.extend_from_slice(&serial_batch);
            }
            serial_batch.clear();
        }

        if gb.bus.ppu.frame_ready() {
            frames += 1;
            gb.bus.ppu.clear_frame_ready();

            if args.verbose && !args.headless {
                let checksum: u64 = gb
                    .bus
                    .ppu
                    .framebuffer()
                    .iter()
                    .fold(0u64, |acc, &px| acc.wrapping_add(px as u64));
                eprintln!("frame {frames} (cycles={cycles}) fb_checksum=0x{checksum:016x}");
            }
        }
    }

    if args.verbose {
        eprintln!("Done: frames={frames} cycles={cycles}");
    }
    if args.print_serial && !args.log_serial && !serial_out.is_empty() {
        print!("{}", String::from_utf8_lossy(&serial_out));
    }

    Ok(0)
}

fn run_suite(args: SuiteArgs) -> Result<i32, String> {
    let mut roms: Vec<PathBuf> = if args.rom_paths.is_empty() {
        discover_roms(&args.rom_dir)?
    } else {
        args.rom_paths
    };
    roms.sort();

    if roms.is_empty() {
        println!("No ROMs found. Use: gb-cli suite --rom-dir <dir>  (or run: gb-cli self-test)");
        return Ok(1);
    }

    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut timeout = 0usize;

    for path in roms {
        let rom = match std::fs::read(&path) {
            Ok(r) => r,
            Err(e) => {
                println!("FAIL {} (read error: {e})", path.display());
                fail += 1;
                continue;
            }
        };
        let cart = match Cartridge::from_rom(rom) {
            Ok(c) => c,
            Err(e) => {
                println!("FAIL {} (invalid ROM: {e:?})", path.display());
                fail += 1;
                continue;
            }
        };

        let (res, serial, frames, cycles) = run_for_serial_result(
            cart,
            args.max_frames,
            args.max_cycles,
            &args.pass_text,
            &args.fail_text,
            args.print_vram,
        );

        match res {
            RomResult::Pass => pass += 1,
            RomResult::Fail => fail += 1,
            RomResult::Timeout => timeout += 1,
        }

        println!(
            "{} {} (frames={frames} cycles={cycles})",
            res.as_str(),
            path.display()
        );

        if args.print_serial && !serial.is_empty() {
            print!("{}", String::from_utf8_lossy(&serial));
            if !serial.ends_with(b"\n") {
                println!();
            }
        }
    }

    println!("Summary: {pass} passed, {fail} failed, {timeout} timed out");

    if fail == 0 && timeout == 0 {
        Ok(0)
    } else {
        Ok(1)
    }
}

fn run_self_test(args: SelfTestArgs) -> Result<i32, String> {
    let rom = make_self_test_rom();
    let cart = Cartridge::from_rom(rom).map_err(|e| format!("invalid ROM: {e:?}"))?;

    let (res, serial, frames, cycles) = run_for_serial_result(
        cart,
        None,
        args.max_cycles,
        &args.pass_text,
        &args.fail_text,
        args.print_vram,
    );

    println!(
        "{} self-test (frames={frames} cycles={cycles})",
        res.as_str()
    );
    if args.print_serial && !serial.is_empty() {
        print!("{}", String::from_utf8_lossy(&serial));
        if !serial.ends_with(b"\n") {
            println!();
        }
    }

    Ok(if res == RomResult::Pass { 0 } else { 1 })
}

fn run() -> Result<i32, String> {
    let cmd = parse_args()?;
    match cmd {
        Command::Run(a) => run_single(a),
        Command::Suite(a) => run_suite(a),
        Command::SelfTest(a) => run_self_test(a),
    }
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("error: {e}");
            print_usage();
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vram_scrape_finds_passed_in_bg_map() {
        let mut vram = [0u8; 0x2000];
        // Write "Passed" into the first row of the 0x9800 BG map.
        let s = b"Passed";
        for (i, &b) in s.iter().enumerate() {
            vram[0x1800 + i] = b;
        }
        let lower = scrape_bg_tilemap_text_lower(&vram, 0x1800);
        assert!(lower.contains("passed"));
    }

    #[test]
    fn vram_scrape_masks_high_bit() {
        let mut vram = [0u8; 0x2000];
        // 0xD0 & 0x7F = 0x50 = 'P'
        vram[0x1800] = 0xD0;
        let lower = scrape_bg_tilemap_text_lower(&vram, 0x1800);
        assert!(lower.starts_with('p'));
    }

    #[test]
    fn vram_scrape_preserves_case() {
        let mut vram = [0u8; 0x2000];
        let s = b"Passed";
        for (i, &b) in s.iter().enumerate() {
            vram[0x1800 + i] = b;
        }
        let t = scrape_bg_tilemap_text(&vram, 0x1800);
        assert!(t.contains("Passed"));
    }
}
