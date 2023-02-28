use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::io::Read;
use std::panic::AssertUnwindSafe;
use log::{error, log, warn};
use winit::event::WindowEvent;
use dbg::BusWrapper;
use mem::{mbc, Wram};
use shared::breakpoints::Breakpoints;
use shared::rom::Rom;
use shared::{io::IO, Events, Ui, egui::Context};
use shared::cpu::Bus;
use shared::winit::window::Window;
use shared::mem::{IOBus, MemoryBus};
use shared::utils::Cell;

use crate::Proxy;
use crate::render::{Event, Render};

mod joy;

pub use joy::Keybindings;

pub struct Emu {
    speed: i32,
    rom: Option<Rom>,
    joy: joy::Joypad,
    pub lcd: lcd::Lcd,
    pub bus: bus::Bus,
    pub cpu: cpu::Cpu,
    pub ppu: ppu::Controller,
    pub mbc: mbc::Controller,
    pub dma: bus::Dma,
    pub timer: bus::Timer,
    running: bool
}

impl Default for Emu {
    fn default() -> Self {
        let lcd = lcd::Lcd::new();
        let mut joy = joy::Joypad::new(Default::default());
        let mut mbc = mbc::Controller::unplugged();
        let mut dma = bus::Dma::default();
        let mut ppu = ppu::Controller::new(false, lcd.clone());
        let mut timer = bus::Timer::default();
        let mut cpu = cpu::Cpu::new(false);
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(false))
            .with_ppu(&mut ppu)
            .configure(&mut dma)
            .configure(&mut timer)
            .configure(&mut cpu)
            .configure(&mut joy);
        Self {
            speed: Default::default(),
            rom: None,
            lcd,
            joy,
            ppu,
            mbc,
            cpu,
            dma,
            bus,
            timer,
            running: false
        }
    }
}

#[derive(Clone)]
pub struct Emulator {
    proxy: Proxy,
    bindings: Rc<RefCell<Keybindings>>,
    breakpoints: Breakpoints,
    emu: Rc<RefCell<Emu>>
}

impl Emulator {

    pub fn new(proxy: Proxy, bindings: Rc<RefCell<Keybindings>>, breakpoints: Breakpoints) -> Self {
        Self { proxy, bindings, emu: Emu::default().cell(), breakpoints }
    }
    pub fn cycle(&mut self, clock: u8) -> bool { self.emu.as_ref().borrow_mut().cycle(clock, &self.breakpoints) }

    pub fn is_running(&self) -> bool { self.emu.as_ref().borrow().running }

    pub fn stop(&mut self) {
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
                let mut emu = Emu::new(self.bindings.clone(), rom.clone(), true);
                self.emu.replace(emu);
                Render::init(self, window);
            },
            Event::UserEvent(Events::Reload) => {
                Render::init(self, window);
            },
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
}

impl dbg::Schedule for Emulator {
    fn breakpoints(&self) -> Breakpoints {
        self.breakpoints.clone()
    }

    fn play(&self) {
        self.emu.as_ref().borrow_mut().running = true;
    }

    fn reset(&self) {
        let emu = self.emu.as_ref().borrow().rom.clone()
            .map(|rom| Emu::new(self.bindings.clone(), rom, false))
            .unwrap_or_else(|| Emu::default());
        self.emu.replace(emu);
        self.proxy.send_event(Events::Reload).ok();
    }

    fn speed(&self) -> i32 { self.emu.as_ref().borrow().speed }
    fn set_speed(&self, speed: i32) { self.emu.as_ref().borrow_mut().speed = speed; }
}

impl Emu {
    // pub const CLOCK_PER_SECOND: u32 = 4_194_304 / 8;
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;
    pub const CYCLE_TIME: f64 = 1.0 / Emu::CLOCK_PER_SECOND as f64;

    pub fn new(bindings: Rc<RefCell<Keybindings>>, rom: Rom, running: bool) -> Self {
        log::info!("starting emu (cgb required: {})", rom.header.kind.requires_gbc());
        let lcd = lcd::Lcd::new();
        let mut joy = joy::Joypad::new(bindings);
        let mut mbc = mem::mbc::Controller::new(&rom);
        let mut dma = bus::Dma::default();
        let mut ppu = ppu::Controller::new(rom.header.kind.requires_gbc(), lcd.clone());
        let mut timer = bus::Timer::default();
        let mut cpu = cpu::Cpu::new(rom.header.kind.requires_gbc());
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(rom.header.kind.requires_gbc()))
            .with_ppu(&mut ppu)
            .configure(&mut dma)
            .configure(&mut timer)
            .configure(&mut cpu)
            .configure(&mut joy);
        timer.offset();
        IOBus::write(&mut bus, IO::BGP as u16, 0xFC); // should be set by BIOS
        IOBus::write(&mut bus, IO::OBP0 as u16, 0xFF); // should be set by BIOS
        IOBus::write(&mut bus, IO::OBP1 as u16, 0xFF); // should be set by BIOS
        IOBus::write(&mut bus, IO::LCDC as u16, 0x91); // should be set by BIOS
        Self {
            speed: Default::default(),
            joy,
            lcd,
            bus,
            cpu,
            ppu,
            mbc,
            dma,
            timer,
            rom: Some(rom),
            running,
        }
    }

    pub fn cycle(&mut self, clock: u8, bp: &Breakpoints) -> bool {
        if !self.running { return false; }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.joy.tick();
            if clock == 0 { // OR clock == 2 && cpu.double_speed()
                self.bus.tick(); // TODO maybe move bus tick in cpu. easier to handle double speed (cause it affects the bus)
                self.cpu.cycle(&mut self.bus);
            }
            self.dma.tick(&mut self.bus);
            self.timer.tick();
            self.ppu.tick();
            self.running &= bp.tick(&self.cpu);
            self.cpu.reset_finished();
        })) {
            Ok(_) => {},
            Err(e) => {
                error!("{e:?}");
                self.running = false;
                return false;
            }
        }
        true
    }
}

impl BusWrapper for Emu {
    fn bus(&self) -> Box<&dyn dbg::Bus> { Box::new(&self.bus) }
    fn mbc(&self) -> &mem::mbc::Controller { &self.mbc }
}
