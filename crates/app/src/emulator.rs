use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::cell::{RefCell};
use std::io::Read;
use std::panic::AssertUnwindSafe;
use log::{error, log, warn};
use winit::event::WindowEvent;
use lcd::Lcd;
use mem::{mbc, Vram, Wram};

use shared::rom::Rom;
use shared::cpu::*;
use shared::{Break, Target, Ui};
use shared::winit::{window::Window};
use shared::mem::MemoryBus;
use crate::{Events, Proxy};
use crate::render::{Render, Event};

pub struct Emu {
    rom: Option<Rom>,
    pub lcd: lcd::Lcd,
    pub bus: bus::Bus,
    pub cpu: core::Cpu,
    pub ppu: ppu::Controller,
    pub mbc: mbc::Controller,
    running: bool,
    breakpoints: Vec<Break>,
}

impl Default for Emu {
    fn default() -> Self {
        let mut mbc = mbc::Controller::unplugged();
        let lcd = Lcd::default();
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(false))
            .with_vram(Vram::new(false));
        let mut cpu = core::Cpu::new(Target::GB);
        let mut ppu = ppu::Controller::new(false, &lcd);
        Self {
            rom: None,
            lcd,
            bus,
            cpu,
            ppu,
            mbc,
            breakpoints: vec![],
            running: false
        }
    }
}

#[derive(Clone)]
pub struct Emulator {
    emu: Rc<RefCell<Emu>>
}

impl Emulator {

    pub fn new() -> Self {
        let emu = Rc::new(RefCell::new(Emu::default()));
        Self { emu }
    }
    pub fn cycle(&mut self, clock: u8) -> bool { self.emu.as_ref().borrow_mut().cycle(clock) }

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
}

impl dbg::Schedule for Emulator {
    fn schedule_break(&mut self, bp: Break) -> &mut Self {
        self.emu.as_ref().borrow_mut().breakpoints.push(bp); self
    }

    fn pause(&mut self) {
        if self.emu.as_ref().borrow().running {
            self.schedule_break(Break::Instructions(1));
        }
    }

    fn play(&mut self) {
        self.emu.as_ref().borrow_mut().running = true;
    }

    fn reset(&mut self) {
        warn!("RESET");
        let running = self.emu.as_ref().borrow().running;
        let emu = self.emu.as_ref().borrow().rom.clone()
            .map(|rom| Emu::new(rom, running))
            .unwrap_or_else(|| Emu::default());
        self.emu.replace(emu);
    }
}

impl Emu {
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;
    pub const CYCLE_TIME: f64 = 1.0 / Emu::CLOCK_PER_SECOND as f64;

    pub fn new(rom: Rom, running: bool) -> Self {
        let lcd = Lcd::default();
        let mut mbc = mem::mbc::Controller::new(&rom);
        let mut ppu = ppu::Controller::new(rom.header.kind.requires_gbc(), &lcd);
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(rom.header.kind.requires_gbc()))
            .with_ppu(&mut ppu);
        let mut cpu = core::Cpu::new(Target::GB);
        Self {
            lcd,
            bus,
            cpu,
            ppu,
            mbc,
            rom: Some(rom),
            breakpoints: vec![],
            running
        }
    }

    pub fn cycle(&mut self, clock: u8) -> bool {
        if !self.running { return false; }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.ppu.tick();
            if clock == 0 { // OR clock == 2 && cpu.double_speed()
                self.bus.tick(); // TODO maybe move bus tick in cpu. easier to handle double speed (cause it affects the bus)
                self.cpu.cycle(&mut self.bus);
                if self.cpu.just_finished {
                    self.running = self.breakpoints.drain_filter(|x| x.tick(&self.cpu)).next().is_none();
                }
            }
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
