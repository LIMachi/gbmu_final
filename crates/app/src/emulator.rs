use std::cmp::Ordering;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use winit::event::WindowEvent;

use bus::Devices;
use mem::{Oam, Vram};
use serial::{Link, Port};
use serial::com::Serial;
use shared::{Events, Handle};
use shared::audio_settings::AudioSettings;
use shared::breakpoints::Breakpoints;
use shared::cpu::Bus;
use shared::emulator::{ReadAccess, Schedule};
use shared::emulator::BusWrapper;
use shared::input::{Keybindings, KeyCat, Shortcut};
use shared::mem::{IOBus, MBCController};
use shared::rom::Rom;
use shared::utils::clock::Clock;
use shared::utils::image::RawData;
use shared::utils::palette::Palette;
use shared::winit::window::Window;

use crate::{AppConfig, Proxy};
use crate::app::RomConfig;
use crate::render::{Event, Render};
use crate::settings::Mode;

#[derive(Default, Serialize, Deserialize)]
pub struct Console {
    speed: i32,
    rom: Option<Rom>,
    pub bus: bus::Bus,
    pub gb: Devices,
    running: bool,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub console: Vec<u8>,
    pub preview: RawData,
    #[serde(skip)]
    pub cover: Option<String>,
    pub path: PathBuf,
    pub ts: String,
}

impl State {
    pub fn load(&self) -> Option<Console> {
        bincode::deserialize(&self.console).ok()
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.ts.eq(&other.ts)
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.ts.partial_cmp(&self.ts)
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.ts.cmp(&self.ts)
    }
}

impl Console {
    pub fn load_state<P: AsRef<Path>>(path: P) -> Option<Self> {
        File::open(path).and_then(|file| {
            bincode::deserialize_from(file).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, format!("{e:?}")))
        }).map_err(|x| log::warn!("error loading state: {x:?}")).ok()
    }

    pub fn active(&self) -> bool { self.rom.is_some() }
}

fn autosave_default() -> u64 { 900 }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmuSettings {
    pub host: String,
    pub port: String,
    #[serde(default)]
    pub palette: Palette,
    #[serde(default = "autosave_default")]
    pub timer: u64,
    #[serde(default)]
    pub autosave: bool,
    #[serde(skip)]
    pub autosave_cycles: usize,
}

impl Default for EmuSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: "27542".to_string(),
            palette: Palette::GrayScale,
            timer: 900,
            autosave: false,
            autosave_cycles: 0,
        }
    }
}

pub struct Emulator {
    pub roms: RomConfig,
    pub proxy: Proxy,
    pub audio: apu::Controller,
    pub settings: EmuSettings,
    pub audio_settings: AudioSettings,
    pub bindings: Keybindings,
    pub(crate) breakpoints: Breakpoints,
    pub console: Console,
    pub cgb: Mode,
    pub bios: bool,
    pub link: Link,
    pub link_port: u16,
    pub timer: Instant,
    clock: Clock,
    pub last: Option<State>,
    throttle: Instant,
}

impl Emulator {
    const AUTOSAVE_CHECK: usize = Console::CLOCK_PER_SECOND as usize * 10;

    pub fn new(proxy: Proxy, conf: AppConfig) -> Self {
        let link = Link::new();
        let port = link.port;
        let mut emu = Self {
            link,
            link_port: port,
            proxy,
            bindings: conf.keys,
            roms: conf.roms,
            settings: conf.emu,
            console: Console::default(),
            audio_settings: conf.audio_settings,
            audio: apu::Controller::new(&conf.sound_device),
            breakpoints: Breakpoints::new(conf.debug.breaks, conf.debug.and),
            cgb: conf.mode,
            bios: conf.bios,
            timer: Instant::now(),
            clock: Clock::new(4),
            last: None,
            throttle: Instant::now(),
        };
        emu.bindings.init();
        emu
    }

    pub fn save_state(&mut self) {
        if self.throttle.elapsed().as_secs_f32() < 0.5 { return; }
        let rom = self.console.rom.as_ref().unwrap();
        let (time, path) = AppConfig::save_path(&rom.header.title);
        let v = bincode::serialize(&self.console).expect("cannot serialize Console");
        let buf = self.console.gb.lcd.pixels.as_ref().unwrap().frame().to_owned();
        let preview = RawData { w: 160, h: 144, data: buf }.downsize([8, 0], [152, 144]);
        let mut h = File::create(&path).expect(format!("cannot open path {path:?}").as_str());
        let state = State {
            console: v,
            preview,
            cover: None,
            path,
            ts: time,
        };
        let v = bincode::serialize(&state).expect("failed to save state");
        h.write_all(&v).expect("failed to save state");
        self.last = Some(state);
        self.throttle = Instant::now();
    }

    pub fn load_state(&mut self, state: Option<&State>) {
        if self.throttle.elapsed().as_secs_f32() < 0.5 { return; }
        if let Some(mut console) = state
            .or(self.last.as_ref())
            .and_then(|x| x.load()) {
            log::info!("loaded state, will save to : {:?}", console.bus.mbc().save_path());
            self.console.bus.save(false);
            self.serial_claim();
            console.gb.serial = Port::new(self.link.port());
            self.audio.reload(&mut console.gb.apu);
            self.console = console;
            self.proxy.send_event(Events::Reload).ok();
            self.proxy.send_event(Events::Open(Handle::Game)).ok();
            self.throttle = Instant::now();
        }
    }

    pub fn mode(&self) -> Mode { self.cgb }

    pub fn link_do<R, F: Fn(&mut Serial) -> R>(&mut self, f: F) -> R {
        self.link.as_mut()
            .map(|x| f(x))
            .unwrap_or_else(|| {
                f(self.console.gb.serial.link())
            })
    }

    pub fn enabled_boot(&self) -> bool {
        self.bios
    }

    pub fn serial_port(&mut self) -> serial::com::Serial {
        self.link.port()
    }
    pub fn serial_claim(&mut self) {
        if self.link.borrowed() {
            self.link.store(self.console.gb.serial.disconnect());
        }
    }

    pub fn cycle(&mut self) {
        if self.is_running() {
            let clock = self.clock.tick();
            self.console.cycle(clock, bus::Settings {
                breakpoints: &mut self.breakpoints,
                sound: &mut self.audio_settings,
            });
            if self.settings.autosave {
                self.settings.autosave_cycles += 1;
                if self.settings.autosave_cycles > Emulator::AUTOSAVE_CHECK {
                    self.settings.autosave_cycles = 0;
                    if self.timer.elapsed().as_secs() > self.settings.timer {
                        self.timer = Instant::now();
                        self.console.bus.save(true);
                    }
                }
            }
        }
    }

    pub fn is_running(&self) -> bool { self.console.running && self.console.rom.is_some() }

    pub fn stop(&mut self, save: bool) {
        self.serial_claim();
        self.link_do(|x| { x.disconnect(); });
        if save { self.console.bus.save(false); }
        self.console = Console::default();
    }

    pub fn cycle_time(&self) -> f64 {
        self.console.speed_mult() * Console::CYCLE_TIME
    }

    fn insert(&mut self, rom: Rom, running: bool) {
        self.serial_claim();
        self.console.bus.save(false);
        self.console = Console::new(self, rom, running);
        self.proxy.send_event(Events::Reload).ok();
        self.timer = Instant::now();
    }
}

#[derive(Default)]
pub struct Screen {
    focus: bool,
}

impl Render for Screen {
    fn init(&mut self, window: &Window, emu: &mut Emulator) {
        emu.console.gb.lcd.init(window);
        window.focus_window();
        self.focus = true;
    }

    fn render(&mut self, emu: &mut Emulator) {
        emu.console.gb.lcd.render();
    }

    fn resize(&mut self, w: u32, h: u32, emu: &mut Emulator) {
        emu.console.gb.lcd.resize(w, h);
    }

    fn handle(&mut self, event: &Event, window: &Window, emu: &mut Emulator) {
        match event {
            Event::UserEvent(Events::Play(rom)) => {
                emu.insert(rom.clone(), true);
                window.set_title(&rom.header.title);
            }
            Event::UserEvent(Events::Reload) => { Render::init(self, window, emu); }
            Event::WindowEvent { window_id, event } if window_id == &window.id() => {
                match event {
                    WindowEvent::CloseRequested => emu.stop(true),
                    WindowEvent::Focused(focus) => self.focus = *focus,
                    _ => {}
                }
            }
            Event::UserEvent(Events::Press(KeyCat::Game(key))) => {
                match key {
                    Shortcut::Quit => {
                        emu.stop(false);
                        emu.proxy.send_event(Events::Close(Handle::Game)).ok();
                    }
                    Shortcut::Save => emu.console.bus.save(false),
                    Shortcut::SpeedUp => emu.speedup(),
                    Shortcut::SpeedDown => emu.speeddown(),
                    Shortcut::SaveState => if emu.console.active() { emu.save_state() },
                    Shortcut::LoadState => emu.load_state(None)
                }
            }
            e => emu.bindings.update(&mut emu.console.gb.joy, e, emu.console.bus.io_regs()),
        }
    }
}

impl ReadAccess for Emulator {
    fn cpu_register(&self, reg: shared::cpu::Reg) -> shared::cpu::Value {
        self.console.gb.cpu.registers().read(reg)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.console.bus.get_range(st, len)
    }

    fn bus(&self) -> Box<&dyn shared::emulator::Bus> {
        Box::<&dyn shared::emulator::Bus>::new(&self.console.bus)
    }

    fn mbc(&self) -> Box<&dyn MBCController> {
        self.console.bus.mbc()
    }
}

impl Schedule for Emulator {
    fn breakpoints(&mut self) -> &mut Breakpoints { &mut self.breakpoints }

    fn play(&mut self) {
        self.console.running = true;
    }

    fn reset(&mut self) {
        let Some(rom) = self.console.rom.clone() else { return; };
        self.insert(rom, false);
    }

    fn speed(&self) -> i32 { self.console.speed }
    fn speedup(&mut self) {
        let speed = self.console.speed + 1;
        if speed <= 15 { self.console.set_speed(speed); }
    }

    fn speeddown(&mut self) {
        let speed = self.console.speed - 1;
        if speed >= -6 { self.console.set_speed(speed); }
    }
}

impl Console {
    #[cfg(feature = "debug")]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 2;
    #[cfg(not(feature = "debug"))]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;

    pub const CYCLE_TIME: f64 = 1.0 / Console::CLOCK_PER_SECOND as f64;

    pub fn new(controller: &mut Emulator, rom: Rom, running: bool) -> Self {
        let cgb = controller.mode().is_cgb();
        let skip = !controller.enabled_boot();
        let gb = Devices::builder()
            .skip_boot(skip)
            .set_cgb(cgb)
            .with_link(controller.serial_port())
            .with_sound_driver(&controller.audio)
            .build();
        let bus = bus::Bus::init(&rom)
            .cgb(cgb)
            .skip_boot(skip)
            .palette(controller.settings.palette)
            .build();
        log::info!("cartridge: {} | device: {}", rom.header.title, if cgb { "CGB" } else { "DMG" });
        Self {
            speed: Default::default(),
            rom: Some(rom),
            running,
            gb,
            bus,
        }
    }

    fn speed_mult(&self) -> f64 {
        match self.speed {
            0 => 1.,
            n @ 1.. => 1. / (1. + 0.2 * n as f64),
            n => (1 << -n) as f64,
        }
    }

    fn set_speed(&mut self, speed: i32) {
        self.speed = speed;
        self.gb.apu.set_speed(self.speed_mult());
    }

    pub fn cycle(&mut self, clock: u8, settings: bus::Settings) {
        self.running = self.bus.tick(&mut self.gb, clock, settings);
    }

    pub fn name(&self) -> &str {
        self.rom.as_ref().map(|x| x.header.title.as_ref()).unwrap_or("GBMU")
    }
}

impl BusWrapper for Console {
    fn bus(&self) -> Box<&dyn shared::emulator::Bus> { Box::new(&self.bus) }
    fn mbc(&self) -> Box<&dyn MBCController> { self.bus.mbc() }
}

impl ppu::VramAccess for Emulator {
    fn vram(&self) -> &Vram { self.console.bus.vram() }

    fn vram_mut(&mut self) -> &mut Vram { self.console.bus.vram_mut() }

    fn oam(&self) -> &Oam {
        self.console.bus.oam()
    }

    fn oam_mut(&mut self) -> &mut Oam {
        self.console.bus.oam_mut()
    }
}

impl ppu::PpuAccess for Emulator {
    fn ppu(&self) -> &ppu::Ppu {
        self.console.gb.ppu.inner()
    }
    fn ppu_mut(&mut self) -> &mut ppu::Ppu {
        self.console.gb.ppu.inner_mut()
    }
}
