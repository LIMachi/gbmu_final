use std::borrow::BorrowMut;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use serde::{Deserialize, Serialize};
use winit::event::{VirtualKeyCode, WindowEvent};
use bus::Devices;
use dbg::BusWrapper;
use mem::{Oam, Vram};
use serial::com::Serial;
use serial::Link;
use shared::breakpoints::Breakpoints;
use shared::rom::Rom;
use shared::{Events, Ui, egui::Context};
use shared::cpu::Bus;
use shared::winit::window::Window;
use shared::mem::{IOBus, MBCController, MemoryBus};
use shared::utils::{ToBox, Cell};

use crate::{AppConfig, Proxy};
use crate::render::{Event, Render};

use shared::audio_settings::AudioSettings;
use shared::emulator::BusWrapper;
use shared::input::{Keybindings, Section};
use crate::settings::{Settings, Mode};

pub struct Console {
    speed: i32,
    rom: Option<Rom>,
    pub bus: bus::Bus,
    pub gb: Devices,
    running: bool
}

impl Default for Console {
    fn default() -> Self {
        Self {
            speed: Default::default(),
            rom: None,
            bus,
            running: false,
            gb: Devices::builder().build()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmuSettings {
    pub host: String,
    pub port: String
}

impl Default for EmuSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: "27542".to_string()
        }
    }
}

#[derive(Clone)]
pub struct Emulator {
    pub proxy: Proxy,
    pub audio: apu::Controller,
    pub settings: Rc<RefCell<EmuSettings>>,
    pub audio_settings: AudioSettings,
    pub bindings: Keybindings,
    breakpoints: Breakpoints,
    pub console: Console,
    pub cgb: Mode,
    pub bios: bool,
    pub link: Rc<RefCell<Link>>,
    pub link_port: u16,
}

impl Emulator {

    pub fn new(proxy: Proxy, bindings: Keybindings, conf: &AppConfig) -> Self {
        let link = Link::new();
        let port = link.port;
        Self {
            link: link.cell(),
            link_port: port,
            proxy,
            bindings,
            settings: conf.emu.clone().cell(),
            console: Console::default(),
            audio_settings: conf.audio_settings.clone(),
            audio: apu::Controller::new(&conf.sound_device),
            breakpoints: Breakpoints::new(conf.debug.breaks.clone()),
            cgb: conf.mode,
            bios: conf.bios,
        }
    }

    pub fn settings(&self) -> Settings {
        Settings::new(self.clone())
    }

    pub fn mode(&self) -> Mode {
        *self.cgb
    }

    pub fn link_do<R, F: Fn(&mut Serial) -> R>(&mut self, f: F) -> R {
        self.link.as_mut()
            .map(|x| f(x))
            .unwrap_or_else(|| {
                f(self.console.gb.serial.link())
            })
    }

    pub fn enabled_boot(&self) -> bool {
        *self.bios
    }

    pub fn serial_port(&self) -> serial::com::Serial {
        RefCell::borrow_mut(&self.link).port()
    }

    pub fn cycle(&mut self, clock: u8) { self.console.cycle(clock, &self.breakpoints); }

    pub fn is_running(&self) -> bool { self.console.running }

    pub fn stop(&mut self) {
        // TODO drop serial first ?
        self.console.replace(Console::default());
    }

    pub fn cycle_time(&self) -> f64 {
        match self.console.speed {
            0 => Console::CYCLE_TIME,
            1 => Console::CYCLE_TIME / 2.,
            n if n < 0 => Console::CYCLE_TIME * ((1 << -n) as f64),
            _ => unimplemented!()
        }
    }

    fn insert(&self, rom: Rom, running: bool) {
        {
            let mut link = RefCell::borrow_mut(&self.link);
            if link.borrowed() {
                link.store(self.console.take().gb.serial.disconnect());
            }
        }
        let emu = Console::new(&self, rom, running);
        self.console.replace(emu);
        self.proxy.send_event(Events::Reload).ok();
    }
}

pub struct Screen;

impl Render for Screen {
    fn init(&mut self, window: &Window, emu: &mut Emulator) {
        log::info!("init LCD");
        self.console.lcd.init(window);
    }
    fn render(&mut self, emu: &mut Emulator) {
        self.console.gb.lcd.render();
    }

    fn resize(&mut self, w: u32, h: u32, emu: &mut Emulator) {
        self.console.gb.lcd.resize(w, h);
    }

    fn handle(&mut self, event: &Event, window: &Window, emu: &mut Emulator) {
        match event {
            Event::UserEvent(Events::Play(rom)) => { self.insert(rom.clone(), true); },
            Event::UserEvent(Events::Reload) => { Render::init(self, window); },
            Event::WindowEvent { window_id, event } if window_id == &window.id() => {
                if event == &WindowEvent::CloseRequested { self.stop(); }
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    self.console.gb.joy.handle(*input);
                }
            },
            _ => {}
        }
    }
}

impl dbg::ReadAccess for Emulator {
    fn cpu_register(&self, reg: shared::cpu::Reg) -> shared::cpu::Value {
        self.console.gb.cpu.registers().read(reg)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.console.bus.get_range(st, len)
    }

    fn bus(&self) -> Box<&dyn BusWrapper> {
        self.console.bus.boxed()
    }

    fn mbc(&self) -> Box<&dyn MBCController> {
        self.console.bus.mbc()
    }

    fn binding(&self, key: VirtualKeyCode) -> Option<Section> {
        self.bindings.get(key)
    }
}

impl dbg::Schedule for Emulator {
    fn breakpoints(&self) -> Breakpoints {
        self.breakpoints.clone()
    }

    fn play(&mut self) {
        self.console.running = true;
    }

    fn reset(&self) {
        let Some(rom) = self.console.rom.clone() else { return };
        self.insert(rom, false);
    }

    fn speed(&self) -> i32 { self.console.speed }
    fn set_speed(&mut self, speed: i32) { self.console.speed = speed; }
}

impl Console {
    // pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 8;
    #[cfg(feature = "debug")]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 2;
    #[cfg(not(feature = "debug"))]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;

    pub const CYCLE_TIME: f64 = 1.0 / Console::CLOCK_PER_SECOND as f64;

    pub fn new(controller: &Emulator, rom: Rom, running: bool) -> Self {
        let cgb = controller.mode().is_cgb();
        let skip = !controller.enabled_boot();
        let mut gb = Console::builder()
            .skip_boot(skip)
            .set_cgb(cgb)
            .with_link(controller.serial_port())
            .with_apu(controller.audio.apu(controller.audio_settings.clone()))
            .with_keybinds(controller.bindings.clone())
            .build();
        let mbc = mem::mbc::Controller::new(&rom);
        let mbc = if skip { mbc.skip_boot() } else { mbc };
        let bus = bus::Bus::new(cgb);
        let mut bus = if skip { bus.skip_boot(if cgb { rom.raw()[0x143] } else { 0 }) } else { bus }
            .with_mbc(mbc)
            .with_ppu(&mut gb.ppu);
        log::info!("cartridge: {} | device: {}", rom.header.title, if cgb { "CGB" } else { "DMG" });
        Self {
            speed: Default::default(),
            rom: Some(rom),
            running,
            gb,
            bus
        }
    }

    pub fn cycle(&mut self, clock: u8, bp: &Breakpoints) {
        if !self.running { return }
        self.bus.tick(&mut self.gb, clock, bp);
    }
}

impl BusWrapper for Console {
    fn bus(&self) -> Box<&dyn dbg::Bus> { Box::new(&self.bus) }
    fn mbc(&self) -> Box<&dyn MBCController> { self.bus.mbc() }
}

impl ppu::render::MemAccess for Emulator {
    fn vram(&self) -> &Vram {
        self.console.bus.
    }

    fn vram_mut(&mut self) -> &mut Vram {
        todo!()
    }

    fn oam(&self) -> &Oam {
        todo!()
    }

    fn oam_mut(&mut self) -> &mut Oam {
        todo!()
    }
}
