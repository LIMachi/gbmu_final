use std::fs::File;
use std::io::Read;
use log::LevelFilter;

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

    pub fn tick(&mut self) {
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

impl core::Bus for FakeBus {
    fn status(&self) -> core::MemStatus {
        self.status
    }

    fn update(&mut self, status: core::MemStatus) {
        self.status = status;
    }
}

fn main() {
    env_logger::init();
    let mut v = Vec::new();
    let mut file = File::open("roms/29459/29459.gbc").expect("not found");
    file.read_to_end(&mut v).expect("failed to read");
    let mut bus = FakeBus::new(v);
    let mut cpu = core::Cpu::new(core::Target::GB);

    loop {
        bus.tick();
        cpu.cycle(&mut bus);
    }
}
