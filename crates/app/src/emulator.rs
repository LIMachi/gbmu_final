use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::io::Read;
use std::panic::AssertUnwindSafe;
use log::{error, log, warn};
use winit::event::WindowEvent;
use bus::Dma;
use dbg::BusWrapper;
use lcd::Lcd;
use mem::{mbc, Vram, Wram};
use mem::mbc::Controller;
use shared::breakpoints::Breakpoints;

use shared::rom::Rom;
use shared::cpu::*;
use shared::io::IO;
use shared::Ui;
use shared::winit::{window::Window};
use shared::mem::{IOBus, MemoryBus};
use shared::utils::Cell;
use crate::{Events, Proxy};
use crate::render::{Render, Event};

pub struct Emu {
    rom: Option<Rom>,
    pub lcd: lcd::Lcd,
    pub bus: bus::Bus,
    pub cpu: core::Cpu,
    pub ppu: ppu::Controller,
    pub mbc: mbc::Controller,
    pub dma: bus::Dma,
    pub timer: bus::Timer,
    running: bool
}

impl Default for Emu {
    fn default() -> Self {
        let lcd = Lcd::new();
        let mut mbc = mbc::Controller::unplugged();
        let mut dma = Dma::default();
        let mut ppu = ppu::Controller::new(false, lcd.clone());
        let mut timer = bus::Timer::default();
        let mut cpu = core::Cpu::new(false);
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(false))
            .with_ppu(&mut ppu)
            .configure(&mut dma)
            .configure(&mut timer)
            .configure(&mut cpu);
        Self {
            rom: None,
            lcd,
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
    breakpoints: Breakpoints,
    emu: Rc<RefCell<Emu>>
}

impl Emulator {

    pub fn new(breakpoints: Breakpoints) -> Self {
        Self { emu: Emu::default().cell(), breakpoints }
    }
    pub fn cycle(&mut self, clock: u8) -> bool { self.emu.as_ref().borrow_mut().cycle(clock, &self.breakpoints) }

    pub fn is_running(&self) -> bool { self.emu.as_ref().borrow().running }

    pub fn stop(&mut self) {
        self.emu.replace(Emu::default());
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

    fn handle(&mut self, event: &Event, _: &Proxy, window: &Window) {
        match event {
            Event::UserEvent(Events::Play(rom)) => {
                let mut emu = Emu::new(rom.clone(), true);
                self.emu.replace(emu);
                self.init(window);
            },
            Event::WindowEvent { window_id, event } if window_id == &window.id() && event == &WindowEvent::CloseRequested => {
                self.stop();
            },
            _ => {}
        }
    }
}

impl dbg::ReadAccess for Emulator {
    fn cpu_register(&self, reg: Reg) -> Value {
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
        warn!("RESET");
        let emu = self.emu.as_ref().borrow().rom.clone()
            .map(|rom| Emu::new(rom, false))
            .unwrap_or_else(|| Emu::default());
        self.emu.replace(emu);
    }
}

impl Emu {
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;
    pub const CYCLE_TIME: f64 = 1.0 / Emu::CLOCK_PER_SECOND as f64;

    pub fn new(rom: Rom, running: bool) -> Self {
        log::info!("starting emu (cgb required: {})", rom.header.kind.requires_gbc());
        let lcd = Lcd::new();
        let mut mbc = mem::mbc::Controller::new(&rom);
        let mut dma = Dma::default();
        let mut ppu = ppu::Controller::new(rom.header.kind.requires_gbc(), lcd.clone());
        let mut timer = bus::Timer::default();
        let mut cpu = core::Cpu::new(rom.header.kind.requires_gbc());
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(rom.header.kind.requires_gbc()))
            .with_ppu(&mut ppu)
            .configure(&mut dma)
            .configure(&mut timer)
            .configure(&mut cpu);
        timer.offset();
        IOBus::write(&mut bus, IO::BGP as u16, 0xFC); // should be set by BIOS
        IOBus::write(&mut bus, IO::OBP0 as u16, 0xFF); // should be set by BIOS
        IOBus::write(&mut bus, IO::OBP1 as u16, 0xFF); // should be set by BIOS
        IOBus::write(&mut bus, IO::LCDC as u16, 0x91); // should be set by BIOS
        Self {
            lcd,
            bus,
            cpu,
            ppu,
            mbc,
            dma,
            timer,
            rom: Some(rom),
            running
        }
    }

    pub fn cycle(&mut self, clock: u8, bp: &Breakpoints) -> bool {
        if !self.running { return false; }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            if clock == 0 { // OR clock == 2 && cpu.double_speed()
                self.bus.tick(); // TODO maybe move bus tick in cpu. easier to handle double speed (cause it affects the bus)
                self.cpu.cycle(&mut self.bus);
            }
            self.timer.tick();
            self.dma.tick(&mut self.bus);
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
    fn mbc(&self) -> &Controller { &self.mbc }
}
