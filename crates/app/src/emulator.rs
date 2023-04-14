use serde::{Deserialize, Serialize};
use winit::event::{VirtualKeyCode, WindowEvent};
use apu::SoundConfig;
use bus::Devices;
use mem::{Oam, Vram};
use serial::com::Serial;
use serial::Link;
use shared::breakpoints::Breakpoints;
use shared::rom::Rom;
use shared::Events;
use shared::cpu::Bus;
use shared::winit::window::Window;
use shared::mem::{IOBus, MBCController};
use shared::emulator::{ReadAccess, Schedule};

use crate::{AppConfig, Proxy};
use crate::render::{Event, Render};

use shared::audio_settings::AudioSettings;
use shared::emulator::BusWrapper;
use shared::input::{Keybindings, Section};
use crate::app::RomConfig;
use crate::settings::Mode;

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
            bus: bus::Bus::new(false),
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

pub struct Emulator {
    pub roms: RomConfig,
    pub proxy: Proxy,
    pub audio: apu::Controller,
    pub settings: EmuSettings,
    pub audio_settings: AudioSettings,
    pub audio_device: SoundConfig,
    pub bindings: Keybindings,
    pub(crate) breakpoints: Breakpoints,
    pub console: Console,
    pub cgb: Mode,
    pub bios: bool,
    pub link: Link,
    pub link_port: u16,
}

impl Emulator {

    pub fn new(proxy: Proxy, conf: AppConfig) -> Self {
        let link = Link::new();
        let port = link.port;
        Self {
            link,
            link_port: port,
            proxy,
            bindings: conf.keys,
            roms: conf.roms,
            settings: conf.emu,
            console: Console::default(),
            audio_settings: conf.audio_settings,
            audio: apu::Controller::new(&conf.sound_device),
            audio_device: conf.sound_device,
            breakpoints: Breakpoints::new(conf.debug.breaks),
            cgb: conf.mode,
            bios: conf.bios,
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

    pub fn enabled_boot(&self) -> bool { self.bios }

    pub fn serial_port(&mut self) -> serial::com::Serial {
        self.link.port()
    }

    pub fn cycle(&mut self, clock: u8) { self.console.cycle(clock, bus::Settings {
        breakpoints: &mut self.breakpoints,
        sound: &mut self.audio_settings,
    }); }

    pub fn is_running(&self) -> bool { self.console.running }

    pub fn stop(&mut self) {
        // TODO drop serial first ?
        self.console = Console::default();
    }

    pub fn cycle_time(&self) -> f64 {
        match self.console.speed {
            0 => Console::CYCLE_TIME,
            1 => Console::CYCLE_TIME / 2.,
            n if n < 0 => Console::CYCLE_TIME * ((1 << -n) as f64),
            _ => unimplemented!()
        }
    }

    fn insert(&mut self, rom: Rom, running: bool) {
        {
            if self.link.borrowed() {
                self.link.store(self.console.gb.serial.disconnect());
            }
        }
        self.console = Console::new(self, rom, running);
        self.proxy.send_event(Events::Reload).ok();
    }
}

#[derive(Default)]
pub struct Screen;

impl Render for Screen {
    fn init(&mut self, window: &Window, emu: &mut Emulator) {
        log::info!("init LCD");
        emu.console.gb.lcd.init(window);
    }
    fn render(&mut self, emu: &mut Emulator) {
         emu.console.gb.lcd.render();
    }

    fn resize(&mut self, w: u32, h: u32, emu: &mut Emulator) {
         emu.console.gb.lcd.resize(w, h);
    }

    fn should_redraw(&self, emu: &mut Emulator) -> bool {
        emu.console.gb.lcd.request()
    }

    fn handle(&mut self, event: &Event, window: &Window, emu: &mut Emulator) {
        match event {
            Event::UserEvent(Events::Play(rom)) => {
                emu.insert(rom.clone(), true);
                if let Some(raw) = &rom.raw {
                    window.set_window_icon(raw.icon());
                }
                window.set_title("GBMU");
            },
            Event::UserEvent(Events::Reload) => { Render::init(self, window, emu); },
            Event::WindowEvent { window_id, event } if window_id == &window.id() => {
                if event == &WindowEvent::CloseRequested { emu.stop(); }
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    emu.console.gb.joy.handle(*input);
                }
            },
            _ => {}
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

    fn binding(&self, key: VirtualKeyCode) -> Option<Section> {
        self.bindings.get(key)
    }
}

impl Schedule for Emulator {
    fn breakpoints(&mut self) -> &mut Breakpoints { &mut self.breakpoints }

    fn play(&mut self) {
        self.console.running = true;
    }

    fn reset(&mut self) {
        let Some(rom) = self.console.rom.clone() else { return };
        self.insert(rom, false);
    }

    fn speed(&self) -> i32 { self.console.speed }
    fn set_speed(&mut self, speed: i32) {
        self.console.speed = speed;
        let time = self.cycle_time();
        log::info!("CY: {time} / CPS: {}", 1f64 / time);
    }
}

impl Console {
    // pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 8;
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
            .with_apu(controller.audio.apu())
            .with_keybinds(controller.bindings.clone())
            .build();
        let mbc = mem::mbc::Controller::new(&rom);
        let mbc = if skip { mbc.skip_boot() } else { mbc };
        let bus = bus::Bus::new(cgb);
        let bus = if skip { bus.skip_boot(if cgb { rom.raw()[0x143] } else { 0 }) } else { bus }
            .with_mbc(mbc);
        log::info!("cartridge: {} | device: {}", rom.header.title, if cgb { "CGB" } else { "DMG" });
        Self {
            speed: Default::default(),
            rom: Some(rom),
            running,
            gb,
            bus
        }
    }

    pub fn cycle(&mut self, clock: u8, settings: bus::Settings) {
        if !self.running { return }
        self.bus.tick(&mut self.gb, clock, settings);
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
}
