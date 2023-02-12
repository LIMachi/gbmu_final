use std::borrow::{Borrow, BorrowMut};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use std::io::Read;
use std::panic::AssertUnwindSafe;
use log::{error, warn};
use mem::{mbc, Vram, Wram};

use shared::rom::Rom;
use shared::cpu::*;
use shared::{Break, Target, Ui};
use shared::winit::{window::Window, event::Event};
use shared::mem::MemoryBus;

pub struct FakeBus {
    ram: Vec<u8>,
    rom: Vec<u8>,
    status: MemStatus
}

impl FakeBus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            ram: vec![0; u16::MAX as usize + 1],
            rom,
            status: MemStatus::ReqRead(0x100u16)
        }
    }
}

impl Bus for FakeBus {
    fn status(&self) -> MemStatus {
        self.status
    }

    fn update(&mut self, status: MemStatus) {
        self.status = status;
    }

    fn tick(&mut self) {
        self.status = match self.status {
            MemStatus::ReqRead(addr) => {
                MemStatus::Read(self.rom[addr as usize])
            },
            MemStatus::ReqWrite(addr) => {
                MemStatus::Write(addr)
            },
            st => st
        }
    }

    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        let st = start as usize;
        let end = st + (len as usize);
        self.rom[st..end].to_vec()
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }
}

pub struct Emu {
    pub lcd: lcd::Lcd,
    pub bus: bus::Bus,
    pub cpu: core::Cpu,
    pub mbc: mbc::Controller,
    running: bool,
    breakpoints: Vec<Break>,
}

impl Default for Emu {
    fn default() -> Self { Emu::new(Emu::POKEMON) }
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
}

impl<E> shared::Render<E> for Emulator {
    fn init(&mut self, window: &Window) {
        self.emu.as_ref().borrow_mut().lcd.init(window);
    }
    fn render(&mut self) {
        self.emu.as_ref().borrow_mut().lcd.render();
    }

    fn resize(&mut self, w: u32, h: u32) {
        self.emu.as_ref().borrow_mut().lcd.resize(w, h);
    }

    fn handle(&mut self, event: &Event<E>) {

    }
}

impl dbg::ReadAccess for Emulator {
    fn cpu_register(&self, reg: Reg) -> Value {
        self.emu.as_ref().borrow().cpu.registers().read(reg)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        use shared::cpu::Bus;
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
        let mut emu = Emu::default();
        emu.running = running;
        self.emu.replace(emu);
    }
}

impl Emu {
    pub const CLOCK_PER_SECOND: u32 = 4_194_304;
    pub const CYCLE_TIME: f64 = 1.0 / Emu::CLOCK_PER_SECOND as f64;

    pub const POKEMON: &'static str = "roms/29459/29459.gbc";

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        let rom = Rom::load(path.as_ref()).expect("failed to load rom");
        let mut mbc = mem::mbc::Controller::new(&rom);
        let mut bus = bus::Bus::new()
            .with_mbc(&mut mbc)
            .with_wram(Wram::new(rom.header.kind.requires_gbc()))
            .with_vram(Vram::new(rom.header.kind.requires_gbc()));
        let mut cpu = core::Cpu::new(Target::GB);
        Self {
            lcd: lcd::Lcd::default(),
            bus,
            cpu,
            mbc,
            breakpoints: vec![],
            running: false
        }
    }

    pub fn cycle(&mut self, clock: u8) -> bool {
        if !self.running { return false; }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            use shared::cpu::Bus;
            if clock == 0 { // OR clock == 2 && cpu.double_speed()
                self.bus.tick();
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
