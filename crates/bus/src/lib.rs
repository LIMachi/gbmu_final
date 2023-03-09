use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use mem::Hram;
use shared::{cpu::MemStatus, mem::*};
use shared::io::{IO, IOReg};
use shared::utils::Cell;

mod io;
mod dma;
mod timer;

pub use dma::Dma;
pub use timer::Timer;

pub struct Empty {}
impl Mem for Empty {}

pub struct Bus {
    rom: Rc<RefCell<dyn Mem>>,
    srom: Rc<RefCell<dyn Mem>>,
    vram: Rc<RefCell<dyn Mem>>,
    sram: Rc<RefCell<dyn Mem>>,
    ram: Rc<RefCell<dyn Mem>>,
    echo: Rc<RefCell<dyn Mem>>, // Fuck off
    oam: Rc<RefCell<dyn Mem>>,
    hram: Rc<RefCell<dyn Mem>>,
    un_1: Rc<RefCell<dyn Mem>>,
    io: io::IORegs,
    ie: IOReg,
    status: MemStatus
}

impl Bus {
    pub fn new(cgb: bool) -> Self {
        Self {
            io: io::IORegs::init(cgb),
            rom: Empty { }.cell(),
            srom: Empty { }.cell(),
            sram: Empty { }.cell(),
            vram: Empty { }.cell(),
            ram: Empty { }.cell(),
            echo: Empty { }.cell(),
            oam: Empty { }.cell(),
            hram: Hram::new().cell(),
            un_1: Empty { }.cell(),
            ie: IOReg::with_access(IO::IE.access()),
            status: MemStatus::ReqRead(0x100)
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM..=ROM_END => self.rom.borrow().read(addr - ROM, addr),
            SROM..=SROM_END => self.srom.borrow().read(addr - SROM, addr),
            VRAM..=VRAM_END => self.vram.borrow().read(addr - VRAM, addr),
            SRAM..=SRAM_END => self.sram.borrow().read(addr - SRAM, addr),
            RAM..=RAM_END => self.ram.borrow().read(addr - RAM, addr),
            ECHO..=ECHO_END => self.ram.borrow().read(addr - ECHO, addr),
            OAM..=OAM_END => self.oam.borrow().read(addr - OAM, addr),
            UN_1..=UN_1_END => self.un_1.borrow().read(addr - UN_1, addr),
            IO..=IO_END => self.io.read(addr - IO, addr),
            HRAM..=HRAM_END => self.hram.borrow().read(addr - HRAM, addr),
            END => self.ie.read(),
            _=> unreachable!()
        }
    }

    fn value(&self, addr: u16) -> u8 {
        match addr {
            ROM..=ROM_END => self.rom.borrow().value(addr - ROM, addr),
            SROM..=SROM_END => self.srom.borrow().value(addr - SROM, addr),
            VRAM..=VRAM_END => self.vram.borrow().value(addr - VRAM, addr),
            SRAM..=SRAM_END => self.sram.borrow().value(addr - SRAM, addr),
            RAM..=RAM_END => self.ram.borrow().value(addr - RAM, addr),
            ECHO..=ECHO_END => self.ram.borrow().value(addr - ECHO, addr),
            OAM..=OAM_END => self.oam.borrow().value(addr - OAM, addr),
            UN_1..=UN_1_END => self.un_1.borrow().value(addr - UN_1, addr),
            IO..=IO_END => self.io.value(addr - IO, addr),
            HRAM..=HRAM_END => self.hram.borrow().value(addr - HRAM, addr),
            END => self.ie.read(),
            _=> unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            ROM..=ROM_END => self.rom.as_ref().borrow_mut().write(addr - ROM, value, addr),
            SROM..=SROM_END => self.srom.as_ref().borrow_mut().write(addr - SROM, value, addr),
            VRAM..=VRAM_END => self.vram.as_ref().borrow_mut().write(addr - VRAM, value, addr),
            SRAM..=SRAM_END => self.sram.as_ref().borrow_mut().write(addr - SRAM, value, addr),
            RAM..=RAM_END => self.ram.as_ref().borrow_mut().write(addr - RAM, value, addr),
            ECHO..=ECHO_END => self.ram.as_ref().borrow_mut().write(addr - ECHO, value, addr),
            OAM..=OAM_END => self.oam.as_ref().borrow_mut().write(addr - OAM, value, addr),
            UN_1..=UN_1_END => self.un_1.as_ref().borrow_mut().write(addr - UN_1, value, addr),
            IO..=IO_END => self.io.write(addr - IO, value, addr),
            HRAM..=HRAM_END => self.hram.as_ref().borrow_mut().write(addr - HRAM, value, addr),
            END => { self.ie.write(0, value, addr) },
            e => unreachable!("wrote address {e:#06X}")
        }
    }
}

impl MemoryBus for Bus {
    fn with_mbc<C: MBCController>(mut self, controller: &mut C) -> Self {
        self.rom = controller.rom();
        self.srom = controller.srom();
        self.sram = controller.sram();
        controller.configure(&mut self);
        self
    }

    fn with_ppu<P: PPU>(mut self, ppu: &mut P) -> Self {
        ppu.configure(&self);
        self.vram = ppu.vram();
        self.oam = ppu.oam();
        self
    }

    fn with_wram<R: Device + Mem + 'static>(mut self, mut wram: R) -> Self {
        wram.configure(&mut self);
        self.ram = wram.cell();
        self
    }
}

impl IOBus for Bus {
    fn io(&self, io: IO) -> IOReg {
        match io {
            IO::IE => self.ie.clone(),
            io => self.io.io(io)
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn value(&self, addr: u16) -> u8 {
        self.value(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.write(addr, value)
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
        };
    }

    #[cfg(feature = "doctor")]
    fn direct_read(&self, offset: u16) -> u8 {
        self.read(offset)
    }

    /// Debug function
    /// will return a range starting from start and up to len bytes long, if possible.
    /// Will end early if the underlying memory range is smaller.
    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        match start {
            ROM..=ROM_END => self.rom.as_ref().borrow_mut().get_range(start, len),
            SROM..=SROM_END => self.srom.as_ref().borrow_mut().get_range(start, len),
            VRAM..=VRAM_END => self.vram.as_ref().borrow_mut().get_range(start, len),
            SRAM..=SRAM_END => self.sram.as_ref().borrow_mut().get_range(start, len),
            RAM..=RAM_END => self.ram.as_ref().borrow_mut().get_range(start, len),
            OAM..=OAM_END => self.oam.as_ref().borrow_mut().get_range(start, len),
            UN_1..=UN_1_END => self.un_1.as_ref().borrow_mut().get_range(start, len),
            IO..=IO_END => self.io.get_range(start, len),
            HRAM..=HRAM_END => self.hram.as_ref().borrow_mut().get_range(start, len),
            END => self.rom.as_ref().borrow_mut().get_range(start, len),
            ECHO..=ECHO_END => self.ram.as_ref().borrow_mut().get_range(start - ECHO + RAM, len),
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }
}
