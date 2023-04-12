use std::rc::Rc;
use std::cell::{Ref, RefCell};
use serde::{Deserialize, Serialize};
use winit::event::{VirtualKeyCode, WindowEvent};
use dbg::BusWrapper;
use mem::mbc;
use serial::com::Serial;
use serial::Link;
use shared::breakpoints::Breakpoints;
use shared::rom::Rom;
use shared::{Events, Ui, egui::Context};
use shared::cpu::Bus;
use shared::winit::window::Window;
use shared::mem::{IOBus, MBCController, MemoryBus};
use shared::utils::Cell;

use crate::{AppConfig, Proxy};
use crate::render::{Event, Render};

mod joy;

use shared::audio_settings::AudioSettings;
use shared::input::{Keybindings, Section};
use crate::settings::{Settings, Mode};

pub struct Emu {
    speed: i32,
    rom: Option<Rom>,
    joy: joy::Joypad,
    pub lcd: lcd::Lcd,
    pub bus: bus::Bus,
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Controller,
    pub dma: ppu::Dma,
    pub hdma: ppu::Hdma,
    pub timer: bus::Timer,
    pub apu: apu::Apu,
    pub serial: serial::Port,
    running: bool
}

impl Default for Emu {
    fn default() -> Self {
        let lcd = lcd::Lcd::new();
        let mbc = mbc::Controller::unplugged();
        let mut ppu = ppu::Controller::new(lcd.clone());

        let joy = joy::Joypad::new(Default::default());
        let serial = serial::Port::new(Serial::phantom());
        let dma = ppu::Dma::default();
        let hdma = ppu::Hdma::default();
        let timer = bus::Timer::default();
        let cpu = cpu::Cpu::new();
        let apu = apu::Apu::default();
        let bus = bus::Bus::new(false)
            .with_mbc(mbc)
            .with_ppu(&mut ppu);
        Self {
            apu,
            speed: Default::default(),
            rom: None,
            lcd,
            joy,
            ppu,
            cpu,
            dma,
            hdma,
            bus,
            timer,
            serial,
            running: false
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
    pub emu: Rc<RefCell<Emu>>,
    pub cgb: Rc<RefCell<Mode>>,
    pub bios: Rc<RefCell<bool>>,
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
            emu: Emu::default().cell(),
            audio_settings: conf.audio_settings.clone(),
            audio: apu::Controller::new(&conf.sound_device),
            breakpoints: Breakpoints::new(conf.debug.breaks.clone()),
            cgb: conf.mode.cell(),
            bios: conf.bios.cell(),
        }
    }

    pub fn settings(&self) -> Settings {
        Settings::new(self.clone())
    }

    pub fn mode(&self) -> Mode {
        *self.cgb.as_ref().borrow()
    }

    pub fn link_do<R, F: Fn(&mut Serial) -> R>(&self, f: F) -> R {
        self.link.as_ref().borrow_mut().as_mut()
            .map(|x| f(x))
            .unwrap_or_else(|| {
                f(self.emu.as_ref().borrow_mut().serial.link())
            })
    }

    pub fn enabled_boot(&self) -> bool {
        *self.bios.as_ref().borrow()
    }

    pub fn serial_port(&self) -> serial::com::Serial {
        RefCell::borrow_mut(&self.link).port()
    }

    pub fn cycle(&mut self, clock: u8) { self.emu.as_ref().borrow_mut().cycle(clock, &self.breakpoints); }

    pub fn is_running(&self) -> bool { self.emu.as_ref().borrow().running }

    pub fn stop(&mut self) {
        // TODO drop serial first ?
        self.emu.replace(Emu::default());
    }

    pub fn cycle_time(&self) -> f64 {
        match self.emu.as_ref().borrow().speed {
            0 => Emu::CYCLE_TIME,
            1 => Emu::CYCLE_TIME / 2.,
            n if n < 0 => Emu::CYCLE_TIME * ((1 << -n) as f64),
            _ => unimplemented!()
        }
    }

    fn insert(&self, rom: Rom, running: bool) {
        {
            let mut link = RefCell::borrow_mut(&self.link);
            if link.borrowed() {
                link.store(self.emu.take().serial.disconnect());
            }
        }
        let emu = Emu::new(&self, rom, running);
        self.emu.replace(emu);
        self.proxy.send_event(Events::Reload).ok();
    }
}

impl Render for Emulator {
    fn init(&mut self, window: &Window) {
        log::info!("init LCD");
        self.emu.as_ref().borrow_mut().lcd.init(window);
    }
    fn render(&mut self) {
        self.emu.as_ref().borrow_mut().lcd.render();
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.emu.as_ref().borrow_mut().lcd.resize(w, h);
    }

    fn handle(&mut self, event: &Event, window: &Window) {
        match event {
            Event::UserEvent(Events::Play(rom)) => {
                self.insert(rom.clone(), true);
                if let Some(raw) = &rom.raw {
                    window.set_window_icon(raw.icon());
                }
                window.set_title(self.emu.as_ref().borrow().name());
            },
            Event::UserEvent(Events::Reload) => { Render::init(self, window); },
            Event::WindowEvent { window_id, event } if window_id == &window.id() => {
                if event == &WindowEvent::CloseRequested { self.stop(); }
                if let WindowEvent::KeyboardInput { input, .. } = event {
                    self.emu.as_ref().borrow_mut().joy.handle(*input);
                }
            },
            _ => {}
        }
    }
}

impl Ui for Emulator {
    fn init(&mut self, ctx: &mut Context) {
        self.emu.as_ref().borrow_mut().ppu.init(ctx);
    }

    fn draw(&mut self, ctx: &mut Context) {
        self.emu.as_ref().borrow_mut().ppu.draw(ctx);
    }

    fn handle(&mut self, event: &shared::Event) {
        self.emu.as_ref().borrow_mut().ppu.handle(event);
    }
}

impl dbg::ReadAccess for Emulator {
    fn cpu_register(&self, reg: shared::cpu::Reg) -> shared::cpu::Value {
        self.emu.as_ref().borrow().cpu.registers().read(reg)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.emu.as_ref().borrow().bus.get_range(st, len)
    }

    fn bus(&self) -> Ref<dyn BusWrapper> {
        self.emu.as_ref().borrow()
    }

    fn binding(&self, key: VirtualKeyCode) -> Option<Section> {
        self.bindings.get(key)
    }
}

impl dbg::Schedule for Emulator {
    fn breakpoints(&self) -> Breakpoints {
        self.breakpoints.clone()
    }

    fn play(&self) {
        self.emu.as_ref().borrow_mut().running = true;
    }

    fn reset(&self) {
        let Some(rom) = self.emu.as_ref().borrow().rom.clone() else { return };
        self.insert(rom, false);
    }

    fn speed(&self) -> i32 { self.emu.as_ref().borrow().speed }
    fn set_speed(&self, speed: i32) { self.emu.as_ref().borrow_mut().speed = speed; }
}

impl Emu {
    // pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 8;
    #[cfg(feature = "debug")]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 2;
    #[cfg(not(feature = "debug"))]
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;

    pub const CYCLE_TIME: f64 = 1.0 / Emu::CLOCK_PER_SECOND as f64;

    pub fn new(controller: &Emulator, rom: Rom, running: bool) -> Self {
        let skip_boot = !controller.enabled_boot();
        let cgb = controller.mode().is_cgb();
        let mut joy = joy::Joypad::new(controller.bindings.clone());
        let mut timer = bus::Timer::default();
        let mut dma = ppu::Dma::default();
        let mut hdma = ppu::Hdma::default();
        let mut apu = controller.audio.apu(controller.audio_settings.clone());
        let mut serial = serial::Port::new(controller.serial_port());

        let lcd = lcd::Lcd::new();
        let mut ppu = ppu::Controller::new(lcd.clone());

        let mbc = mem::mbc::Controller::new(&rom);
        let mut cpu = cpu::Cpu::new();
        let bus = bus::Bus::new(cgb);
        let mbc = if skip_boot { mbc.skip_boot() } else { mbc };
        let bus = if skip_boot { bus.skip_boot(if cgb { rom.raw()[0x143] } else { 0 }) } else { bus }
            .with_mbc(mbc)
            .with_ppu(&mut ppu)
            .configure(&mut dma)
            .configure(&mut hdma)
            .configure(&mut timer)
            .configure(&mut cpu)
            .configure(&mut joy)
            .configure(&mut apu)
            .configure(&mut serial);
        log::info!("cartridge: {} | device: {}", rom.header.title, if cgb { "CGB" } else { "DMG" });
        if skip_boot { cpu.skip_boot(cgb); timer.offset() };
        Self {
            speed: Default::default(),
            joy,
            lcd,
            bus,
            cpu,
            ppu,
            dma,
            hdma,
            timer,
            serial,
            rom: Some(rom),
            running,
            apu
        }
    }

    pub fn cycle(&mut self, clock: u8, bp: &Breakpoints) {
        if !self.running { return }
        self.joy.tick();
        let tick = self.hdma.tick(&mut self.bus);
        self.bus.tick();
        if clock == 0 || (clock == 2 && self.cpu.double_speed()) {
            self.serial.tick();
            self.timer.tick();
            self.dma.tick(&mut self.bus);
            if !tick { self.cpu.cycle(&mut self.bus); }
        }
        self.ppu.tick();
        self.apu.tick(self.cpu.double_speed());
        self.running &= bp.tick(&self.cpu, self.bus.last());
        self.cpu.reset_finished();
    }

    pub fn name(&self) -> &str {
        self.rom.as_ref().map(|x| x.header.title.as_ref()).unwrap_or("GBMU")
    }
}

impl BusWrapper for Emu {
    fn bus(&self) -> Box<&dyn dbg::Bus> { Box::new(&self.bus) }
    fn mbc(&self) -> Ref<dyn MBCController> { self.bus.mbc() }
}
