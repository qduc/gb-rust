use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::cpu::Flag;
use gb_core::cpu::Cpu;

fn make_rom(program: &[u8]) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x0147] = 0x00; // ROM only
    rom[0x0148] = 0x00; // 32KB
    rom[0x0149] = 0x00; // No RAM
    rom[..program.len()].copy_from_slice(program);
    rom
}

fn setup(program: &[u8]) -> (Cpu, Bus) {
    let cart = Cartridge::from_rom(make_rom(program)).unwrap();
    (Cpu::new(), Bus::new(cart))
}

fn assert_flags(cpu: &Cpu, z: bool, n: bool, h: bool, c: bool) {
    assert_eq!(cpu.flag(Flag::Z), z, "Z");
    assert_eq!(cpu.flag(Flag::N), n, "N");
    assert_eq!(cpu.flag(Flag::H), h, "H");
    assert_eq!(cpu.flag(Flag::C), c, "C");
}

#[test]
fn ei_enables_ime_after_following_instruction() {
    let (mut cpu, mut bus) = setup(&[0xFB, 0x00]); // EI ; NOP

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 1);
    assert!(!cpu.ime);
    assert!(cpu.ei_pending);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 2);
    assert!(cpu.ime);
    assert!(!cpu.ei_pending);
}

#[test]
fn di_clears_ei_pending_and_keeps_ime_disabled() {
    let (mut cpu, mut bus) = setup(&[0xFB, 0xF3, 0x00]); // EI ; DI ; NOP

    cpu.step(&mut bus);
    assert!(cpu.ei_pending);

    cpu.step(&mut bus);
    assert!(!cpu.ime);
    assert!(!cpu.ei_pending);

    cpu.step(&mut bus);
    assert!(!cpu.ime);
}

#[test]
fn pending_interrupt_after_ei_is_serviced_on_third_step() {
    let (mut cpu, mut bus) = setup(&[0xFB, 0x00, 0x00]); // EI ; NOP ; NOP
    cpu.sp = 0xFFFE;

    bus.ie = 0x01;
    bus.iflag = 0x01;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 1);
    assert!(!cpu.ime);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 2);
    assert!(cpu.ime);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 20);
    assert_eq!(cpu.pc, 0x0040);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x02);
    assert_eq!(bus.read8(0xFFFD), 0x00);
}

#[test]
fn halt_without_pending_interrupt_stays_halted() {
    let (mut cpu, mut bus) = setup(&[]);
    cpu.halted = true;
    cpu.pc = 0x1234;

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 4);
    assert!(cpu.halted);
    assert_eq!(cpu.pc, 0x1234);
}

#[test]
fn halt_with_pending_interrupt_and_ime_false_resumes_execution() {
    let (mut cpu, mut bus) = setup(&[0x00]); // NOP
    cpu.halted = true;
    cpu.ime = false;

    bus.ie = 0x01;
    bus.iflag = 0x01;

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 4);
    assert!(!cpu.halted);
    assert_eq!(cpu.pc, 0);
    assert_ne!(bus.iflag & 0x01, 0);
}

#[test]
fn halt_with_pending_interrupt_and_ime_true_services_interrupt() {
    let (mut cpu, mut bus) = setup(&[]);
    cpu.halted = true;
    cpu.ime = true;
    cpu.pc = 0x2000;
    cpu.sp = 0xFFFE;

    bus.ie = 0x01;
    bus.iflag = 0x01;

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 20);
    assert!(!cpu.halted);
    assert!(!cpu.ime);
    assert_eq!(cpu.pc, 0x0040);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x00);
    assert_eq!(bus.read8(0xFFFD), 0x20);
}

#[test]
fn jr_and_conditional_jr_update_pc_and_cycles() {
    // JR +2 jumps to LD A,0x42
    let (mut cpu, mut bus) = setup(&[0x18, 0x02, 0x00, 0x00, 0x3E, 0x42]);
    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 12);
    assert_eq!(cpu.pc, 4);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 8);
    assert_eq!(cpu.a, 0x42);
    assert_eq!(cpu.pc, 6);

    // JR NZ not taken when Z is set.
    let (mut cpu, mut bus) = setup(&[0x20, 0x7F]);
    cpu.set_flag(Flag::Z, true);
    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 8);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn call_and_ret_round_trip_pc_and_stack() {
    let (mut cpu, mut bus) = setup(&[0xCD, 0x05, 0x00, 0x00, 0x00, 0xC9]); // CALL 0x0005 ; ... ; RET
    cpu.sp = 0xFFFE;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 24);
    assert_eq!(cpu.pc, 0x0005);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x03);
    assert_eq!(bus.read8(0xFFFD), 0x00);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 16);
    assert_eq!(cpu.pc, 0x0003);
    assert_eq!(cpu.sp, 0xFFFE);
}

#[test]
fn rst_pushes_return_address_and_jumps_to_vector() {
    let (mut cpu, mut bus) = setup(&[0xFF]); // RST 38h
    cpu.sp = 0xFFFE;

    let cycles = cpu.step(&mut bus);

    assert_eq!(cycles, 16);
    assert_eq!(cpu.pc, 0x0038);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0x01);
    assert_eq!(bus.read8(0xFFFD), 0x00);
}

#[test]
fn push_pop_af_masks_lower_flag_nibble() {
    let (mut cpu, mut bus) = setup(&[0xF5, 0x3E, 0x12, 0xF1]); // PUSH AF ; LD A,0x12 ; POP AF
    cpu.sp = 0xFFFE;
    cpu.a = 0xAB;
    cpu.f = 0xF3;

    cpu.step(&mut bus);
    assert_eq!(cpu.sp, 0xFFFC);
    assert_eq!(bus.read8(0xFFFC), 0xF0);
    assert_eq!(bus.read8(0xFFFD), 0xAB);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0x12);

    cpu.step(&mut bus);
    assert_eq!(cpu.a, 0xAB);
    assert_eq!(cpu.f, 0xF0);
}

#[test]
fn cb_rlc_and_bit_hl_update_flags_and_cycles() {
    // RLC B: 0x80 -> 0x01, carry set.
    let (mut cpu, mut bus) = setup(&[0xCB, 0x00]);
    cpu.b = 0x80;
    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 8);
    assert_eq!(cpu.b, 0x01);
    assert_flags(&cpu, false, false, false, true);

    // BIT 0,(HL): checks bit without changing C, and costs 12 cycles for (HL).
    let (mut cpu, mut bus) = setup(&[0xCB, 0x46]);
    cpu.set_hl(0xC000);
    bus.write8(0xC000, 0x00);
    cpu.set_flag(Flag::C, true);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 12);
    assert_flags(&cpu, true, false, true, true);
}

#[test]
fn halt_bug_duplicates_next_opcode_fetch_when_ime_off_and_interrupt_pending() {
    // HALT ; NOP ; NOP
    let (mut cpu, mut bus) = setup(&[0x76, 0x00, 0x00]);
    cpu.ime = false;
    bus.ie = 0x01;
    bus.iflag = 0x01;

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 1);
    assert!(cpu.halted);
    assert_ne!(bus.iflag & 0x01, 0);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    // HALT bug keeps PC on the duplicated fetch.
    assert_eq!(cpu.pc, 1);

    let cycles = cpu.step(&mut bus);
    assert_eq!(cycles, 4);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn cpu_step_advances_timer_without_external_bus_tick() {
    let (mut cpu, mut bus) = setup(&[0x00, 0x00, 0x00, 0x00]); // 4x NOP

    bus.write8(0xFF05, 0x00); // TIMA
    bus.write8(0xFF07, 0x05); // enable timer at 16-cycle period

    for _ in 0..4 {
        let cycles = cpu.step(&mut bus);
        assert_eq!(cycles, 4);
    }

    assert_eq!(bus.read8(0xFF05), 0x01);
}
