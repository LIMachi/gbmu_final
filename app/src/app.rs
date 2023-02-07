use std::cell::{Ref, RefCell};
use std::fs::File;
use std::io::Read;
use std::panic::AssertUnwindSafe;
use std::rc::Rc;
use log::error;

pub struct FakeBus {
    rom: Vec<u8>,
    status: core::MemStatus
}

impl FakeBus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            rom,
            status: core::MemStatus::ReqRead(0x100u16)
        }
    }
}

impl core::Bus for FakeBus {
    fn status(&self) -> core::MemStatus {
        self.status
    }

    fn update(&mut self, status: core::MemStatus) {
        self.status = status;
    }

    fn tick(&mut self) {
        self.status = match self.status {
            core::MemStatus::ReqRead(addr) => {
                core::MemStatus::Read(self.rom[addr as usize])
            },
            core::MemStatus::ReqWrite(addr) => {
                core::MemStatus::Write(addr)
            },
            st => st
        }
    }
}

pub struct Emu {
    bus: FakeBus,
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

    pub fn cpu_register(&self, reg: core::Reg) -> core::Value {
        self.emu.borrow().cpu.registers().read(reg)
    }
}

impl Emu {
    pub fn new() -> Self {
        let mut v = Vec::new();
        let mut file = File::open("roms/29459/29459.gbc").expect("not found");
        file.read_to_end(&mut v).expect("failed to read");
        println!("{:#X?}", &v[0x101..0x104]);
        let mut bus = FakeBus::new(v);
        let mut cpu = core::Cpu::new(core::Target::GB);
        Self {
            bus,
            cpu,
            running: true
        }
    }

    pub fn cycle(&mut self) {
        use core::Bus;
        self.bus.tick();
        if !self.running {
            return ;
        }
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
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