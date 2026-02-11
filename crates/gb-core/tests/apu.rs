use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;

const NR10: u16 = 0xFF10;
const NR52: u16 = 0xFF26;
const WAVE_START: u16 = 0xFF30;

fn make_bus() -> Bus {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00;
    rom[0x0148] = 0x00;
    rom[0x0149] = 0x00;
    let cart = Cartridge::from_rom(rom).expect("valid ROM");
    Bus::new(cart)
}

fn read_mask(addr: u16) -> u8 {
    match addr {
        0xFF10 => 0x80,
        0xFF11 => 0x3F,
        0xFF12 => 0x00,
        0xFF13 => 0xFF,
        0xFF14 => 0xBF,
        0xFF15 => 0xFF,
        0xFF16 => 0x3F,
        0xFF17 => 0x00,
        0xFF18 => 0xFF,
        0xFF19 => 0xBF,
        0xFF1A => 0x7F,
        0xFF1B => 0xFF,
        0xFF1C => 0x9F,
        0xFF1D => 0xFF,
        0xFF1E => 0xBF,
        0xFF1F => 0xFF,
        0xFF20 => 0xFF,
        0xFF21 => 0x00,
        0xFF22 => 0x00,
        0xFF23 => 0xBF,
        0xFF24 => 0x00,
        0xFF25 => 0x00,
        0xFF26 => 0x70,
        0xFF27..=0xFF2F => 0xFF,
        0xFF30..=0xFF3F => 0x00,
        _ => unreachable!("bad APU register {addr:#06X}"),
    }
}

#[test]
fn apu_register_masks_match_dmg_expectations() {
    let mut bus = make_bus();

    for &value in &[0x00, 0x55, 0xFF] {
        for addr in NR10..=0xFF3F {
            if addr != NR52 {
                bus.write8(addr, value);
            }

            let expected = if addr == NR52 {
                bus.read8(addr)
            } else {
                read_mask(addr) | value
            };
            assert_eq!(
                bus.read8(addr),
                expected,
                "addr={addr:#06X} value={value:#04X}"
            );

            bus.write8(0xFF25, 0);
            bus.write8(0xFF1A, 0);
        }
    }
}

#[test]
fn apu_power_off_clears_regs_but_preserves_wave_ram() {
    let mut bus = make_bus();

    for i in 0..16 {
        bus.write8(WAVE_START + i, 0x30 + i as u8);
    }

    for addr in NR10..=0xFF25 {
        bus.write8(addr, 0xFF);
    }

    bus.write8(NR52, 0x00);

    assert_eq!(bus.read8(NR52), 0x70);
    for addr in NR10..=NR52 {
        if addr == NR52 {
            continue;
        }

        let expected = read_mask(addr);
        assert_eq!(bus.read8(addr), expected, "addr={addr:#06X}");
    }

    for i in 0..16 {
        assert_eq!(bus.read8(WAVE_START + i), 0x30 + i as u8);
    }

    bus.write8(NR52, 0x80);
    assert_eq!(bus.read8(NR52), 0xF0);

    for addr in NR10..=0xFF25 {
        assert_eq!(bus.read8(addr), read_mask(addr), "addr={addr:#06X}");
    }
}

#[test]
fn apu_ignores_register_writes_while_powered_off() {
    let mut bus = make_bus();

    bus.write8(NR52, 0x00);
    for addr in NR10..=0xFF25 {
        bus.write8(addr, 0xAA);
    }

    for addr in NR10..=0xFF25 {
        assert_eq!(bus.read8(addr), read_mask(addr), "addr={addr:#06X}");
    }

    bus.write8(WAVE_START, 0x7B);
    assert_eq!(bus.read8(WAVE_START), 0x7B);
}

#[test]
fn apu_trigger_and_length_counter_drive_nr52_status() {
    let mut bus = make_bus();

    bus.write8(0xFF24, 0x77);
    bus.write8(0xFF25, 0x11);

    bus.write8(0xFF12, 0xF0);
    bus.write8(0xFF11, 0x3F);
    bus.write8(0xFF14, 0xC0);

    assert_ne!(bus.read8(NR52) & 0x01, 0);

    bus.tick(8_192);

    assert_eq!(bus.read8(NR52) & 0x01, 0);
}

#[test]
fn apu_emits_interleaved_stereo_samples() {
    let mut bus = make_bus();

    bus.write8(0xFF24, 0x77);
    bus.write8(0xFF25, 0x11);

    bus.write8(0xFF11, 0x80);
    bus.write8(0xFF12, 0xF0);
    bus.write8(0xFF13, 0x00);
    bus.write8(0xFF14, 0x80);

    bus.tick(65_536);

    let samples = bus.apu.take_samples();
    assert!(!samples.is_empty());
    assert_eq!(samples.len() % 2, 0);
    assert!(samples.iter().any(|s| s.abs() > 0.001));
}

#[test]
fn apu_long_run_sample_rate_stays_stable() {
    let mut bus = make_bus();

    bus.write8(0xFF24, 0x77);
    bus.write8(0xFF25, 0x11);
    bus.write8(0xFF11, 0x80);
    bus.write8(0xFF12, 0xF0);
    bus.write8(0xFF13, 0xAA);
    bus.write8(0xFF14, 0x87);

    // 2.0 seconds at DMG CPU clock.
    bus.tick(8_388_608);

    let samples = bus.apu.take_samples();
    assert_eq!(samples.len(), 192_000);
    assert!(samples.iter().all(|s| s.is_finite()));
}
