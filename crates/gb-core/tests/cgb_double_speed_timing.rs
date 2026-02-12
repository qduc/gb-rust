use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;

fn make_rom(cgb_flag: u8, program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[0x0143] = cgb_flag;
    rom[..program.len()].copy_from_slice(program);
    rom
}

#[test]
fn cgb_double_speed_halves_bus_cycles_for_timer_div() {
    // Baseline: in normal speed, 64x NOP (64 * 4 = 256 CPU cycles) increments DIV by 1.
    {
        let cart = Cartridge::from_rom(make_rom(0x80, &[0x00])).unwrap();
        let mut bus = Bus::new(cart);
        let mut cpu = Cpu::new();

        bus.write8(0xFF04, 0x00); // reset DIV
        assert_eq!(bus.read8(0xFF04), 0x00);

        for _ in 0..64 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }
        assert_eq!(bus.read8(0xFF04), 0x01);
    }

    // Double speed: 64x NOP should only advance DIV by half as much (not enough for +1).
    // 128x NOP (128 * 4 CPU cycles) should increment DIV by 1.
    {
        // STOP 00; NOP
        let cart = Cartridge::from_rom(make_rom(0x80, &[0x10, 0x00, 0x00])).unwrap();
        let mut bus = Bus::new(cart);
        let mut cpu = Cpu::new();

        bus.write8(0xFF4D, 0x01); // KEY1 prepare
        let cycles = cpu.step(&mut bus);
        assert_eq!(cycles, 8);
        assert_eq!(bus.read8(0xFF4D) & 0x80, 0x80);

        bus.write8(0xFF04, 0x00); // reset DIV after switching speed
        assert_eq!(bus.read8(0xFF04), 0x00);

        for _ in 0..64 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }
        assert_eq!(bus.read8(0xFF04), 0x00);

        for _ in 0..64 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }
        assert_eq!(bus.read8(0xFF04), 0x01);
    }
}

#[test]
fn cgb_double_speed_does_not_speed_up_apu_frame_sequencer() {
    // We observe APU length clocking via NR52 channel enable.
    // Configure CH1 length = 1 and enable length; it should turn off after the first
    // frame sequencer tick (8192 base cycles).

    const NR11: u16 = 0xFF11;
    const NR12: u16 = 0xFF12;
    const NR13: u16 = 0xFF13;
    const NR14: u16 = 0xFF14;
    const NR52: u16 = 0xFF26;

    fn setup_ch1_length_1(bus: &mut Bus) {
        // Reset + power on to align frame sequencer.
        bus.write8(NR52, 0x00);
        bus.write8(NR52, 0x80);

        // Length = 1 (64 - 63).
        bus.write8(NR11, 0x3F);
        // Enable DAC.
        bus.write8(NR12, 0xF0);
        bus.write8(NR13, 0x00);
        // Trigger (bit7) + length enable (bit6).
        bus.write8(NR14, 0xC0);

        assert_eq!(bus.read8(NR52) & 0x01, 0x01);
    }

    // Normal speed: 2048 NOPs = 8192 CPU cycles = 8192 base cycles.
    {
        let cart = Cartridge::from_rom(make_rom(0x80, &[0x00])).unwrap();
        let mut bus = Bus::new(cart);
        let mut cpu = Cpu::new();

        setup_ch1_length_1(&mut bus);

        for _ in 0..2048 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }

        assert_eq!(bus.read8(NR52) & 0x01, 0x00);
    }

    // Double speed: 2048 NOPs = 8192 CPU cycles = 4096 base cycles (not enough).
    // 4096 NOPs = 16384 CPU cycles = 8192 base cycles (enough to clock length once).
    {
        // STOP 00; NOP
        let cart = Cartridge::from_rom(make_rom(0x80, &[0x10, 0x00, 0x00])).unwrap();
        let mut bus = Bus::new(cart);
        let mut cpu = Cpu::new();

        bus.write8(0xFF4D, 0x01);
        cpu.step(&mut bus);
        assert_eq!(bus.read8(0xFF4D) & 0x80, 0x80);

        setup_ch1_length_1(&mut bus);

        for _ in 0..2048 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }
        assert_eq!(bus.read8(NR52) & 0x01, 0x01);

        for _ in 0..2048 {
            let c = cpu.step(&mut bus);
            assert_eq!(c, 4);
        }
        assert_eq!(bus.read8(NR52) & 0x01, 0x00);
    }
}
