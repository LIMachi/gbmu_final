use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::panic::AssertUnwindSafe;

use log::error;

use shared::cpu::*;
use shared::Target;

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

    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        let st = start as usize;
        let end = st + (len as usize);
        self.rom[st..end].to_vec()
    }
}

pub struct Emu {
    pub bus: FakeBus,
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
        use core::Bus;
        self.emu.as_ref().borrow().bus.get_range(st, len)
    }

}

impl Emu {
    pub fn new() -> Self {
        let mut v = Vec::new();
        let mut file = File::open("roms/29459/29459.gbc").expect("not found");
        file.read_to_end(&mut v).expect("failed to read");
        // println!("{:#X?}", &v[0..0x100]);
        let mut bus = FakeBus::new(v);
        let mut cpu = core::Cpu::new(Target::GB);
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
