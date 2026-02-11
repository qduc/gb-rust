#[cfg(feature = "sdl")]
mod audio;

#[cfg(not(feature = "sdl"))]
fn main() {
    println!("gb-sdl built without SDL support; enable with: cargo run -p gb-sdl --features sdl");
}

#[cfg(feature = "sdl")]
fn keycode_to_button(key: sdl2::keyboard::Keycode) -> Option<gb_core::input::Button> {
    use gb_core::input::Button;
    use sdl2::keyboard::Keycode;

    match key {
        Keycode::Up => Some(Button::Up),
        Keycode::Down => Some(Button::Down),
        Keycode::Left => Some(Button::Left),
        Keycode::Right => Some(Button::Right),
        Keycode::Z => Some(Button::A),
        Keycode::X => Some(Button::B),
        Keycode::Backspace => Some(Button::Select),
        Keycode::Return => Some(Button::Start),
        _ => None,
    }
}

#[cfg(feature = "sdl")]
fn init_dmg_post_boot(gb: &mut gb_core::gb::GameBoy) {
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
    gb.bus.iflag = 0x01;

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

#[cfg(feature = "sdl")]
fn write_framebuffer_rgba8888_bytes(fb: &gb_core::ppu::Framebuffer, out: &mut [u8]) {
    assert_eq!(out.len(), fb.len() * 4);
    for (px, chunk) in fb.iter().zip(out.chunks_exact_mut(4)) {
        let a = (px >> 24) as u8;
        let r = (px >> 16) as u8;
        let g = (px >> 8) as u8;
        let b = *px as u8;
        chunk[0] = r;
        chunk[1] = g;
        chunk[2] = b;
        chunk[3] = a;
    }
}

#[cfg(feature = "sdl")]
fn main() -> Result<(), String> {
    use gb_core::bus::Bus;
    use gb_core::cartridge::Cartridge;
    use gb_core::cpu::Cpu;
    use gb_core::gb::GameBoy;
    use gb_core::ppu::{LCD_HEIGHT, LCD_WIDTH};

    use sdl2::event::Event;
    use sdl2::keyboard::Keycode;
    use sdl2::pixels::PixelFormatEnum;

    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let audio_subsystem = sdl.audio()?;

    let window = video_subsystem
        .window("gb-sdl", (LCD_WIDTH as u32) * 3, (LCD_HEIGHT as u32) * 3)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(LCD_WIDTH as u32, LCD_HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            LCD_WIDTH as u32,
            LCD_HEIGHT as u32,
        )
        .map_err(|e| e.to_string())?;
    let mut framebuffer_bytes = vec![0u8; LCD_WIDTH * LCD_HEIGHT * 4];

    let audio_out = audio::SdlAudio::new(
        &audio_subsystem,
        gb_core::apu::Apu::DEFAULT_SAMPLE_RATE_HZ as i32,
        gb_core::apu::Apu::DEFAULT_CHANNELS,
    )?;

    let rom = if let Some(path) = std::env::args().nth(1) {
        std::fs::read(&path).map_err(|e| e.to_string())?
    } else {
        // Minimal "valid enough" ROM for Cartridge::from_rom (header bytes only).
        let mut rom = vec![0u8; 0x8000];
        rom[0x0147] = 0x00; // ROM only
        rom[0x0148] = 0x00; // 32KiB
        rom[0x0149] = 0x00; // no RAM
        rom
    };

    let cart = Cartridge::from_rom(rom).map_err(|e| format!("{e:?}"))?;
    let mut gb = GameBoy {
        cpu: Cpu::new(),
        bus: Bus::new(cart),
    };
    init_dmg_post_boot(&mut gb);

    let mut event_pump = sdl.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if let Some(btn) = keycode_to_button(key) {
                        gb.bus.set_joypad_button(btn, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(btn) = keycode_to_button(key) {
                        gb.bus.set_joypad_button(btn, false);
                    }
                }

                _ => {}
            }
        }

        gb.run_frame();
        audio::pump_apu_to_sdl(&mut gb.bus.apu, &audio_out)?;
        write_framebuffer_rgba8888_bytes(gb.bus.ppu.framebuffer(), &mut framebuffer_bytes);

        texture
            .update(None, &framebuffer_bytes, LCD_WIDTH * 4)
            .map_err(|e| e.to_string())?;

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .map_err(|e| e.to_string())?;
        canvas.present();
    }

    Ok(())
}

#[cfg(all(feature = "sdl", test))]
mod tests {
    use super::{keycode_to_button, write_framebuffer_rgba8888_bytes};
    use gb_core::input::Button;
    use gb_core::ppu::FRAMEBUFFER_LEN;
    use sdl2::keyboard::Keycode;

    #[test]
    fn keycode_mapping_matches_expected_buttons() {
        assert_eq!(keycode_to_button(Keycode::Up), Some(Button::Up));
        assert_eq!(keycode_to_button(Keycode::Z), Some(Button::A));
        assert_eq!(keycode_to_button(Keycode::Return), Some(Button::Start));
        assert_eq!(keycode_to_button(Keycode::Tab), None);
    }

    #[test]
    fn framebuffer_argb_to_rgba_conversion_is_stable() {
        let mut fb = [0u32; FRAMEBUFFER_LEN];
        fb[0] = 0xFF00_0000; // opaque black
        fb[1] = 0x1122_3344; // A,R,G,B

        let mut bytes = vec![0u8; FRAMEBUFFER_LEN * 4];
        write_framebuffer_rgba8888_bytes(&fb, &mut bytes);

        assert_eq!(&bytes[0..4], &[0x00, 0x00, 0x00, 0xFF]);
        assert_eq!(&bytes[4..8], &[0x22, 0x33, 0x44, 0x11]);
    }
}
