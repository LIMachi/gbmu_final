use std::rc::Rc;
use std::cell::RefCell;
use std::io::Read;
use std::panic::AssertUnwindSafe;
use log::error;

use shared::rom::Rom;
use shared::cpu::*;
use shared::Target;

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
    pub bus: bus::Bus,
    pub cpu: core::Cpu,
    running: bool,
}

#[derive(Clone)]
pub struct Emulator {
    emu: Rc<RefCell<Emu>>
}

impl Emulator {
    pub fn new() -> Self {
        Self { emu: Rc::new(RefCell::new(Emu::new())) }
    }
    pub fn cycle(&mut self) {
        self.emu.borrow_mut().cycle();
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

impl Emu {
    pub fn new() -> Self {
        let rom = Rom::load("roms/29459/29459.gbc").expect("failed to load rom");
        let mbc = mem::mbc::mbc0::Controller::new(&rom);
        let mut bus = bus::Bus::new().with_memory_controller(mbc);
        let mut cpu = core::Cpu::new(Target::GB);
        Self {
            bus,
            cpu,
            running: true
        }
    }

    pub fn cycle(&mut self) {
        use shared::cpu::Bus;
        if !self.running {
            return ;
        }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.bus.tick();
            self.cpu.cycle(&mut self.bus);
        })) {
            Ok(_) => {},
            Err(e) => {
                error!("{e:?}");
                self.running = false;
            }
        }
    }
}
