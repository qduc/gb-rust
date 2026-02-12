mod audio;

use egui::{Context, Key, Modifiers, TopBottomPanel, Window};
use egui_sdl2_gl::painter::Painter;
use egui_sdl2_gl::{with_sdl2, DpiScaling, EguiStateHandler, ShaderVersion};
use gb_core::bus::Bus;
use gb_core::cartridge::Cartridge;
use gb_core::cpu::Cpu;
use gb_core::gb::GameBoy;
use gb_core::ppu::{LCD_HEIGHT, LCD_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::FullscreenType;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const GB_FPS: f64 = 4_194_304.0 / (456.0 * 154.0);
const AUTOSAVE_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TurboMode {
    Normal,
    X2,
    X4,
    Uncapped,
}

impl TurboMode {
    fn speed_multiplier(self) -> Option<u32> {
        match self {
            Self::Normal => Some(1),
            Self::X2 => Some(2),
            Self::X4 => Some(4),
            Self::Uncapped => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Normal => "1x",
            Self::X2 => "2x",
            Self::X4 => "4x",
            Self::Uncapped => "Uncapped",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum QuickSlot {
    Slot1,
    Slot2,
    Slot3,
}

impl QuickSlot {
    fn index(self) -> u8 {
        match self {
            Self::Slot1 => 1,
            Self::Slot2 => 2,
            Self::Slot3 => 3,
        }
    }

    fn all() -> [Self; 3] {
        [Self::Slot1, Self::Slot2, Self::Slot3]
    }
}

struct App {
    gb: GameBoy,
    rom_path: Option<PathBuf>,
    sav_path: Option<PathBuf>,
    state_path: Option<PathBuf>,
    paused: bool,
    turbo: TurboMode,
    volume: f32,
    integer_scale: bool,
    fullscreen: bool,
    auto_pause_on_ui: bool,
    show_audio_settings: bool,
    show_video_settings: bool,
    show_debug_window: bool,
    status: String,
    last_frame_cycles: u64,
    total_frames: u64,
    last_battery_save_at: Instant,
}

impl App {
    fn new() -> Result<Self, String> {
        let gb = Self::default_gameboy()?;
        Ok(Self {
            gb,
            rom_path: None,
            sav_path: None,
            state_path: None,
            paused: false,
            turbo: TurboMode::Normal,
            volume: 1.0,
            integer_scale: true,
            fullscreen: false,
            auto_pause_on_ui: true,
            show_audio_settings: false,
            show_video_settings: false,
            show_debug_window: false,
            status: "Ready".to_string(),
            last_frame_cycles: 0,
            total_frames: 0,
            last_battery_save_at: Instant::now(),
        })
    }

    fn default_gameboy() -> Result<GameBoy, String> {
        let mut rom = vec![0u8; 0x8000];
        rom[0x0147] = 0x00;
        rom[0x0148] = 0x00;
        rom[0x0149] = 0x00;
        let cart = Cartridge::from_rom(rom).map_err(|e| format!("{e:?}"))?;
        let mut gb = GameBoy {
            cpu: Cpu::new(),
            bus: Bus::new(cart),
        };
        init_post_boot(&mut gb);
        Ok(gb)
    }

    fn state_slot_path(&self, slot: QuickSlot) -> Option<PathBuf> {
        self.rom_path.as_ref().map(|rom| {
            let stem = rom
                .file_stem()
                .and_then(|s| s.to_str())
                .filter(|s| !s.is_empty())
                .unwrap_or("rom");
            rom.with_file_name(format!("{stem}.slot{}.state", slot.index()))
        })
    }

    fn save_state(&mut self, path: &Path) -> Result<(), String> {
        let bytes = bincode::serialize(&self.gb)
            .map_err(|e| format!("failed to encode save state: {e}"))?;
        std::fs::write(path, bytes).map_err(|e| format!("failed to write state: {e}"))
    }

    fn load_state(&mut self, path: &Path) -> Result<(), String> {
        let bytes = std::fs::read(path).map_err(|e| format!("failed to read state: {e}"))?;
        let loaded: GameBoy = bincode::deserialize(&bytes)
            .map_err(|e| format!("failed to decode save state: {e}"))?;
        self.gb = loaded;
        Ok(())
    }

    fn battery_save_now(&mut self) {
        if let Some(path) = &self.sav_path {
            if let Err(e) = self.gb.bus.save_to_path(path) {
                self.status = format!("Battery save failed: {e:?}");
            }
            self.last_battery_save_at = Instant::now();
        }
    }

    fn maybe_battery_autosave(&mut self) {
        if self.last_battery_save_at.elapsed() >= AUTOSAVE_INTERVAL {
            self.battery_save_now();
        }
    }

    fn load_rom(&mut self, rom_path: PathBuf) -> Result<(), String> {
        self.battery_save_now();

        let rom = std::fs::read(&rom_path)
            .map_err(|e| format!("failed to read ROM {}: {e}", rom_path.display()))?;
        let cart = Cartridge::from_rom(rom).map_err(|e| format!("invalid ROM: {e:?}"))?;
        let mut gb = GameBoy {
            cpu: Cpu::new(),
            bus: Bus::new(cart),
        };
        init_post_boot(&mut gb);

        let sav_path = rom_path.with_extension("sav");
        let state_path = rom_path.with_extension("state");
        if let Err(e) = gb.bus.load_from_path(&sav_path) {
            self.status = format!("ROM loaded, save load failed: {e:?}");
        }

        self.gb = gb;
        self.rom_path = Some(rom_path.clone());
        self.sav_path = Some(sav_path);
        self.state_path = Some(state_path);
        self.paused = false;
        self.total_frames = 0;
        self.last_frame_cycles = 0;
        self.last_battery_save_at = Instant::now();
        self.status = format!("Loaded {}", rom_path.display());
        Ok(())
    }

    fn ui(
        &mut self,
        ctx: &Context,
        window: &mut sdl2::video::Window,
        gb_texture: egui::TextureId,
        ppp: f32,
    ) -> bool {
        let mut request_open_rom = false;
        let mut request_save_state = false;
        let mut request_load_state = false;
        let mut request_exit = false;
        let mut request_quick_save: Option<QuickSlot> = None;
        let mut request_quick_load: Option<QuickSlot> = None;

        TopBottomPanel::top("menu_top").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        request_open_rom = true;
                        ui.close();
                    }
                    if ui.button("Save State").clicked() {
                        request_save_state = true;
                        ui.close();
                    }
                    if ui.button("Load State").clicked() {
                        request_load_state = true;
                        ui.close();
                    }
                    ui.separator();
                    for slot in QuickSlot::all() {
                        if ui
                            .button(format!("Quick Save Slot {}", slot.index()))
                            .clicked()
                        {
                            request_quick_save = Some(slot);
                            ui.close();
                        }
                        if ui
                            .button(format!("Quick Load Slot {}", slot.index()))
                            .clicked()
                        {
                            request_quick_load = Some(slot);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        request_exit = true;
                        ui.close();
                    }
                });

                ui.menu_button("Emulation", |ui| {
                    if ui
                        .button(if self.paused { "Resume" } else { "Pause" })
                        .clicked()
                    {
                        self.paused = !self.paused;
                        ui.close();
                    }
                    ui.checkbox(&mut self.auto_pause_on_ui, "Auto-pause on UI focus");
                    ui.separator();
                    ui.label("Turbo");
                    for mode in [
                        TurboMode::Normal,
                        TurboMode::X2,
                        TurboMode::X4,
                        TurboMode::Uncapped,
                    ] {
                        ui.radio_value(&mut self.turbo, mode, mode.label());
                    }
                });

                ui.menu_button("Audio", |ui| {
                    if ui.button("Audio Settings...").clicked() {
                        self.show_audio_settings = true;
                        ui.close();
                    }
                    ui.add(egui::Slider::new(&mut self.volume, 0.0..=2.0).text("Volume"));
                });

                ui.menu_button("Video", |ui| {
                    if ui.button("Video Settings...").clicked() {
                        self.show_video_settings = true;
                        ui.close();
                    }
                    ui.checkbox(&mut self.integer_scale, "Integer scaling");
                    if ui.checkbox(&mut self.fullscreen, "Fullscreen").changed() {
                        let mode = if self.fullscreen {
                            FullscreenType::Desktop
                        } else {
                            FullscreenType::Off
                        };
                        if let Err(e) = window.set_fullscreen(mode) {
                            self.status = format!("Fullscreen failed: {e}");
                        }
                    }
                });

                ui.menu_button("Debug", |ui| {
                    ui.checkbox(&mut self.show_debug_window, "Show debug window");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available = ui.available_size();
            let base_w = LCD_WIDTH as f32;
            let base_h = LCD_HEIGHT as f32;
            let scale_x = if base_w > 0.0 {
                available.x / base_w
            } else {
                1.0
            };
            let scale_y = if base_h > 0.0 {
                available.y / base_h
            } else {
                1.0
            };
            let mut scale = scale_x.min(scale_y);
            if self.integer_scale {
                scale = scale.floor().max(1.0);
            }
            let draw_w = (base_w * scale).max(1.0);
            let draw_h = (base_h * scale).max(1.0);
            ui.vertical_centered(|ui| {
                let image = egui::Image::new((gb_texture, egui::vec2(draw_w / ppp, draw_h / ppp)));
                ui.add(image);
            });
        });

        if self.show_audio_settings {
            Window::new("Audio Settings")
                .open(&mut self.show_audio_settings)
                .show(ctx, |ui| {
                    ui.add(egui::Slider::new(&mut self.volume, 0.0..=2.0).text("Volume"));
                });
        }

        if self.show_video_settings {
            Window::new("Video Settings")
                .open(&mut self.show_video_settings)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.integer_scale, "Integer scaling");
                    if ui.checkbox(&mut self.fullscreen, "Fullscreen").changed() {
                        let mode = if self.fullscreen {
                            FullscreenType::Desktop
                        } else {
                            FullscreenType::Off
                        };
                        if let Err(e) = window.set_fullscreen(mode) {
                            self.status = format!("Fullscreen failed: {e}");
                        }
                    }
                });
        }

        if self.show_debug_window {
            let paused = self.paused;
            let turbo = self.turbo.label().to_string();
            let frame_cycles = self.last_frame_cycles;
            let total_frames = self.total_frames;
            let rom_name = self.rom_display_name();
            let status = self.status.clone();
            Window::new("Debug")
                .open(&mut self.show_debug_window)
                .show(ctx, |ui| {
                    ui.label(format!("Paused: {}", paused));
                    ui.label(format!("Turbo: {}", turbo));
                    ui.label(format!("Frame cycles: {}", frame_cycles));
                    ui.label(format!("Frames: {}", total_frames));
                    ui.label(format!("ROM: {}", rom_name));
                    ui.label(format!("Status: {}", status));
                });
        }

        if request_open_rom {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Game Boy ROM", &["gb", "gbc"])
                .pick_file()
            {
                if let Err(e) = self.load_rom(path) {
                    self.status = e;
                }
            }
        }

        if request_save_state {
            if let Some(path) = self.state_path.clone() {
                if let Err(e) = self.save_state(&path) {
                    self.status = e;
                } else {
                    self.status = format!("Saved state to {}", path.display());
                }
            }
        }

        if request_load_state {
            if let Some(path) = self.state_path.clone() {
                if let Err(e) = self.load_state(&path) {
                    self.status = e;
                } else {
                    self.status = format!("Loaded state from {}", path.display());
                }
            }
        }

        if let Some(slot) = request_quick_save {
            if let Some(path) = self.state_slot_path(slot) {
                if let Err(e) = self.save_state(&path) {
                    self.status = e;
                } else {
                    self.status = format!("Quick save slot {}", slot.index());
                }
            }
        }

        if let Some(slot) = request_quick_load {
            if let Some(path) = self.state_slot_path(slot) {
                if let Err(e) = self.load_state(&path) {
                    self.status = e;
                } else {
                    self.status = format!("Quick load slot {}", slot.index());
                }
            }
        }

        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("ROM: {}", self.rom_display_name()));
                ui.separator();
                ui.label(format!(
                    "State: {}",
                    if self.paused { "Paused" } else { "Running" }
                ));
                ui.separator();
                ui.label(format!("Turbo: {}", self.turbo.label()));
                ui.separator();
                ui.label(format!("Volume: {:.0}%", self.volume * 100.0));
                ui.separator();
                ui.label(self.status.clone());
            });
        });

        request_exit
    }

    fn rom_display_name(&self) -> String {
        self.rom_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("<none>")
            .to_string()
    }
}

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

fn init_common_io_post_boot(gb: &mut gb_core::gb::GameBoy) {
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

    init_common_io_post_boot(gb);
}

fn init_cgb_post_boot(gb: &mut gb_core::gb::GameBoy) {
    // CGB (no-boot-rom) register values commonly used by emulators.
    gb.cpu.a = 0x11;
    gb.cpu.f = 0x80;
    gb.cpu.b = 0x00;
    gb.cpu.c = 0x00;
    gb.cpu.d = 0xFF;
    gb.cpu.e = 0x56;
    gb.cpu.h = 0x00;
    gb.cpu.l = 0x0D;
    gb.cpu.sp = 0xFFFE;
    gb.cpu.pc = 0x0100;

    gb.bus.ie = 0x00;
    gb.bus.iflag = 0x01;

    init_common_io_post_boot(gb);

    let cgb_io_inits: &[(u16, u8)] = &[
        (0xFF4D, 0x00), // KEY1
        (0xFF4F, 0x00), // VBK
        (0xFF70, 0x01), // SVBK
        (0xFF68, 0x00), // BCPS
        (0xFF69, 0x00), // BCPD
        (0xFF6A, 0x00), // OCPS
        (0xFF6B, 0x00), // OCPD
    ];
    for &(addr, val) in cgb_io_inits {
        gb.bus.write8(addr, val);
    }

    // CGB boot ROM sets BG palette 0 color 0 to white (0x7FFF).
    // Without this, many CGB games start with a black screen because
    // palette RAM defaults to zero.
    gb.bus.ppu.write_bgpi(0x80); // auto-increment, index 0
    gb.bus.ppu.write_bgpd(0xFF); // low byte of 0x7FFF
    gb.bus.ppu.write_bgpd(0x7F); // high byte of 0x7FFF
}

fn init_post_boot(gb: &mut gb_core::gb::GameBoy) {
    if gb.bus.mode == gb_core::bus::EmulationMode::Cgb {
        init_cgb_post_boot(gb);
    } else {
        init_dmg_post_boot(gb);
    }
}

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

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video_subsystem = sdl.video()?;
    let audio_subsystem = sdl.audio()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let mut window = video_subsystem
        .window("gb-sdl", (LCD_WIDTH as u32) * 3, (LCD_HEIGHT as u32) * 3)
        .position_centered()
        .allow_highdpi()
        .opengl()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let _gl_ctx = window.gl_create_context().map_err(|e| e.to_string())?;
    window
        .subsystem()
        .gl_set_swap_interval(1)
        .map_err(|e| e.to_string())?;

    let (mut painter, mut egui_state): (Painter, EguiStateHandler) =
        with_sdl2(&window, ShaderVersion::Default, DpiScaling::Default);

    let egui_ctx = Context::default();
    let gb_texture = painter.new_user_texture_rgba8(
        (LCD_WIDTH, LCD_HEIGHT),
        vec![0u8; LCD_WIDTH * LCD_HEIGHT * 4],
        false,
    );

    let mut framebuffer_bytes = vec![0u8; LCD_WIDTH * LCD_HEIGHT * 4];

    let audio_out = audio::SdlAudio::new(
        &audio_subsystem,
        gb_core::apu::Apu::DEFAULT_SAMPLE_RATE_HZ as i32,
        gb_core::apu::Apu::DEFAULT_CHANNELS,
    )?;

    let mut app = App::new()?;
    if let Some(path) = std::env::args().nth(1).map(PathBuf::from) {
        if let Err(e) = app.load_rom(path) {
            app.status = e;
        }
    }

    let mut next_frame_at = Instant::now();
    let app_start = Instant::now();
    let mut event_pump = sdl.event_pump()?;
    let mut ui_wants_input = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            egui_state.process_input(&window, event.clone(), &mut painter);

            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keymod,
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    let command = (keymod & sdl2::keyboard::Mod::LGUIMOD
                        == sdl2::keyboard::Mod::LGUIMOD)
                        || (keymod & sdl2::keyboard::Mod::RGUIMOD == sdl2::keyboard::Mod::RGUIMOD)
                        || (keymod & sdl2::keyboard::Mod::LCTRLMOD
                            == sdl2::keyboard::Mod::LCTRLMOD)
                        || (keymod & sdl2::keyboard::Mod::RCTRLMOD
                            == sdl2::keyboard::Mod::RCTRLMOD);

                    if command && key == Keycode::O {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Game Boy ROM", &["gb", "gbc"])
                            .pick_file()
                        {
                            if let Err(e) = app.load_rom(path) {
                                app.status = e;
                            }
                        }
                        continue;
                    }

                    if key == Keycode::P {
                        app.paused = !app.paused;
                        continue;
                    }

                    if command && key == Keycode::S {
                        if let Some(path) = app.state_path.clone() {
                            if let Err(e) = app.save_state(&path) {
                                app.status = e;
                            } else {
                                app.status = format!("Saved state to {}", path.display());
                            }
                        }
                        continue;
                    }

                    if command && key == Keycode::L {
                        if let Some(path) = app.state_path.clone() {
                            if let Err(e) = app.load_state(&path) {
                                app.status = e;
                            } else {
                                app.status = format!("Loaded state from {}", path.display());
                            }
                        }
                        continue;
                    }

                    if key == Keycode::F5 {
                        if let Some(path) = app.state_slot_path(QuickSlot::Slot1) {
                            if let Err(e) = app.save_state(&path) {
                                app.status = e;
                            } else {
                                app.status = "Quick save slot 1".to_string();
                            }
                        }
                        continue;
                    }

                    if key == Keycode::F8 {
                        if let Some(path) = app.state_slot_path(QuickSlot::Slot1) {
                            if let Err(e) = app.load_state(&path) {
                                app.status = e;
                            } else {
                                app.status = "Quick load slot 1".to_string();
                            }
                        }
                        continue;
                    }

                    if !ui_wants_input {
                        if let Some(btn) = keycode_to_button(key) {
                            app.gb.bus.set_joypad_button(btn, true);
                        }
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if !ui_wants_input {
                        if let Some(btn) = keycode_to_button(key) {
                            app.gb.bus.set_joypad_button(btn, false);
                        }
                    }
                }

                _ => {}
            }
        }

        let raw_input = std::mem::take(&mut egui_state.input);
        let mut request_exit = false;
        let full_output = egui_ctx.run(raw_input, |ctx| {
            if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Space)) {
                app.paused = !app.paused;
            }
            request_exit = app.ui(ctx, &mut window, gb_texture, painter.pixels_per_point);
        });
        egui_state.process_output(&window, &full_output.platform_output);
        ui_wants_input = egui_ctx.wants_keyboard_input() || egui_ctx.wants_pointer_input();
        if request_exit {
            break 'running;
        }

        let should_pause = app.paused || (app.auto_pause_on_ui && ui_wants_input);

        let now = Instant::now();
        if let Some(multiplier) = app.turbo.speed_multiplier() {
            let frame_duration = Duration::from_secs_f64(1.0 / (GB_FPS * multiplier as f64));
            if now < next_frame_at {
                std::thread::sleep(next_frame_at - now);
            }
            next_frame_at += frame_duration;
            if next_frame_at < Instant::now() {
                next_frame_at = Instant::now();
            }
        } else {
            next_frame_at = now;
        }

        if !should_pause {
            app.gb.run_frame();
            app.last_frame_cycles = 0;
            app.total_frames = app.total_frames.saturating_add(1);
            app.maybe_battery_autosave();
        } else {
            audio_out.clear();
        }

        audio::pump_apu_to_sdl(&mut app.gb.bus.apu, &audio_out, app.volume)?;
        write_framebuffer_rgba8888_bytes(app.gb.bus.ppu.framebuffer(), &mut framebuffer_bytes);
        painter.update_user_texture_rgba8_data(gb_texture, framebuffer_bytes.clone());

        let clipped = egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        painter.paint_jobs(None, full_output.textures_delta, clipped);
        window.gl_swap_window();

        egui_state.input = egui::RawInput {
            screen_rect: Some(painter.screen_rect),
            time: Some(app_start.elapsed().as_secs_f64()),
            ..Default::default()
        };
        egui_state
            .input
            .viewports
            .entry(egui::ViewportId::ROOT)
            .or_default()
            .native_pixels_per_point = Some(painter.pixels_per_point);

        if should_pause {
            std::thread::sleep(Duration::from_millis(8));
        }
    }

    app.battery_save_now();

    Ok(())
}

#[cfg(test)]
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
