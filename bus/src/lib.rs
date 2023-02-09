use std::cell::RefCell;
use std::rc::Rc;
use shared::{cpu::MemStatus, mem::*};

pub struct Empty {}
impl Mem for Empty {}

pub struct Bus {
    rom: Rc<RefCell<dyn Mem>>,
    srom: Rc<RefCell<dyn Mem>>,
    vram: Rc<RefCell<dyn Mem>>,
    sram: Rc<RefCell<dyn Mem>>,
    ram: Rc<RefCell<dyn Mem>>,
    echo: Rc<RefCell<Empty>>, // Fuck off
    oam: Rc<RefCell<dyn Mem>>,
    io: Rc<RefCell<dyn Mem>>,
    hram: Rc<RefCell<dyn Mem>>,
    un_1: Rc<RefCell<Empty>>,
    status: MemStatus
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Rc::new(RefCell::new(Empty { })),
            srom: Rc::new(RefCell::new(Empty { })),
            sram: Rc::new(RefCell::new(Empty { })),
            vram: Rc::new(RefCell::new(Empty { })),
            ram: Rc::new(RefCell::new(Empty { })),
            echo: Rc::new(RefCell::new(Empty { })),
            oam: Rc::new(RefCell::new(Empty { })),
            io: Rc::new(RefCell::new(Empty { })),
            hram: Rc::new(RefCell::new(Empty { })),
            un_1: Rc::new(RefCell::new(Empty { })),
            status: MemStatus::ReqRead(0x100)
        }
    }

    pub fn with_memory_controller<C: MemoryController>(mut self, controller: C) -> Self {
        controller.register(&mut self);
        self
    }

    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            ROM..=ROM_END => self.rom.borrow().read(addr - ROM, addr),
            SROM..=SROM_END => self.srom.borrow().read(addr - SROM, addr),
            VRAM..=VRAM_END => self.vram.borrow().read(addr - VRAM, addr),
            SRAM..=SRAM_END => self.sram.borrow().read(addr - SRAM, addr),
            RAM..=RAM_END => self.ram.borrow().read(addr - RAM, addr),
            OAM..=OAM_END => self.oam.borrow().read(addr - OAM, addr),
            UN_1..=UN_1_END => self.un_1.borrow().read(addr - UN_1, addr),
            IO..=IO_END => self.io.borrow().read(addr - IO, addr),
            HRAM..=HRAM_END => self.hram.borrow().read(addr - HRAM, addr),
            END => self.rom.borrow().read(addr - END, addr),
            _=> unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            ROM..=ROM_END => self.rom.borrow_mut().write(addr - ROM, value, addr),
            SROM..=SROM_END => self.srom.borrow_mut().write(addr - SROM, value, addr),
            VRAM..=VRAM_END => self.vram.borrow_mut().write(addr - VRAM, value, addr),
            SRAM..=SRAM_END => self.sram.borrow_mut().write(addr - SRAM, value, addr),
            RAM..=RAM_END => self.ram.borrow_mut().write(addr - RAM, value, addr),
            OAM..=OAM_END => self.oam.borrow_mut().write(addr - OAM, value, addr),
            UN_1..=UN_1_END => self.un_1.borrow_mut().write(addr - UN_1, value, addr),
            IO..=IO_END => self.io.borrow_mut().write(addr - IO, value, addr),
            HRAM..=HRAM_END => self.hram.borrow_mut().write(addr - HRAM, value, addr),
            END => self.rom.borrow_mut().write(addr - END, value, addr),
            _=> unreachable!()
        }
    }
}

impl MemoryBus for Bus {
    fn set_rom(&mut self, rom: Rc<RefCell<dyn Mem>>) {
        self.rom = rom;
    }
    fn set_srom(&mut self, srom: Rc<RefCell<dyn Mem>>) {
        self.srom = srom;
    }

    fn set_sram(&mut self, sram: Rc<RefCell<dyn Mem>>) {
        self.sram = sram;
    }
}

impl shared::cpu::Bus for Bus {
    fn status(&self) -> MemStatus {
        self.status
    }

    fn update(&mut self, status: MemStatus) {
        self.status = status;
    }

    fn tick(&mut self) {
        self.status = match self.status {
            MemStatus::ReqRead(addr) => MemStatus::Read(self.read(addr)),
            MemStatus::ReqWrite(addr) => MemStatus::Write(addr),
            st => st
        }
    }

    /// Debug function
    /// will return a range starting from start and up to len bytes long, if possible.
    /// Will end early if the underlying memory range is smaller.
    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        vec![]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}
