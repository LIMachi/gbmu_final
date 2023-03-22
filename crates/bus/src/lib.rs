#![feature(trait_upcasting)]

use std::cell::{Ref, RefCell};
use std::rc::Rc;
use mem::Hram;
use shared::{cpu::MemStatus, cpu::Op, mem::*};
use shared::io::{IO, IOReg};
use shared::utils::Cell;

mod io;
mod timer;

pub use timer::Timer;

pub struct Empty {}
impl Mem for Empty {}

type LockedMem = Lock<Rc<RefCell<dyn Mem>>>;

pub struct Bus {
    mbc: Rc<RefCell<dyn MBCController>>,
    rom: LockedMem,
    srom: LockedMem,
    sram: LockedMem,
    ram: LockedMem,
    hram: LockedMem,
    un_1: LockedMem,
    vram: Rc<RefCell<dyn Mem>>,
    oam: Rc<RefCell<dyn Mem>>,
    io: io::IORegs,
    ie: IOReg,
    status: MemStatus,
    last: Option<Op>,
    cgb: bool,
}

impl Bus {
    pub fn new(cgb: bool, compat: bool) -> Self {
        Self {
            mbc: mem::mbc::Controller::unplugged().cell(),
            cgb,
            last: None,
            io: io::IORegs::init(compat),
            rom: Empty { }.locked_cell(),
            srom: Empty { }.locked_cell(),
            sram: Empty { }.locked_cell(),
            vram: Empty { }.cell(),
            oam: Empty { }.cell(),
            ram: Empty { }.locked_cell(),
            hram: Hram::new().locked_cell(),
            un_1: Empty { }.locked_cell(),
            ie: IOReg::with_access(IO::IE.access()),
            status: MemStatus::ReqRead(0x0)
        }
    }

    pub fn skip_boot(mut self) -> Self {
        self.status = MemStatus::ReqRead(0x100);
        self
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM..=ROM_END => self.rom.read(addr - ROM, addr),
            SROM..=SROM_END => self.srom.read(addr - SROM, addr),
            VRAM..=VRAM_END => self.vram.read(addr - VRAM, addr),
            SRAM..=SRAM_END => self.sram.read(addr - SRAM, addr),
            RAM..=RAM_END => self.ram.read(addr - RAM, addr),
            ECHO..=ECHO_END => self.ram.read(addr - ECHO, addr),
            OAM..=OAM_END => self.oam.read(addr - OAM, addr),
            UN_1..=UN_1_END => self.un_1.read(addr - UN_1, addr),
            IO..=IO_END => self.io.read(addr - IO, addr),
            HRAM..=HRAM_END => self.hram.read(addr - HRAM, addr),
            END => self.ie.read()
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            ROM..=ROM_END => self.rom.write(addr - ROM, value, addr),
            SROM..=SROM_END => self.srom.write(addr - SROM, value, addr),
            VRAM..=VRAM_END => self.vram.write(addr - VRAM, value, addr),
            SRAM..=SRAM_END => self.sram.write(addr - SRAM, value, addr),
            RAM..=RAM_END => self.ram.write(addr - RAM, value, addr),
            ECHO..=ECHO_END => self.ram.write(addr - ECHO, value, addr),
            OAM..=OAM_END => self.oam.write(addr - OAM, value, addr),
            UN_1..=UN_1_END => self.un_1.write(addr - UN_1, value, addr),
            BOOT => self.rom.write(BOOT, value, BOOT),
            IO..=IO_END => self.io.write(addr - IO, value, addr),
            HRAM..=HRAM_END => self.hram.write(addr - HRAM, value, addr),
            END => { self.ie.write(0, value, addr) }
        }
    }

    pub fn last(&mut self) -> Option<Op> {
        self.last.take()
    }
}

impl MemoryBus for Bus {
    fn with_mbc<C: MBCController + 'static>(mut self, mut controller: C) -> Self {
        controller.configure(&mut self);
        self.mbc = controller.cell();
        self.rom = Lock::new(self.mbc.clone());
        self.srom = Lock::new(self.mbc.clone());
        self.sram = Lock::new(self.mbc.clone());
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
        self.ram = wram.locked_cell();
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

    fn is_cgb(&self) -> bool { self.cgb }

    fn read_with(&self, addr: u16, source: Source) -> u8 {
        match addr {
            ROM..=ROM_END => self.rom.read_with(addr - ROM, addr, source),
            SROM..=SROM_END => self.srom.read_with(addr - SROM, addr, source),
            VRAM..=VRAM_END => self.vram.read_with(addr - VRAM, addr, source),
            SRAM..=SRAM_END => self.sram.read_with(addr - SRAM, addr, source),
            RAM..=RAM_END => self.ram.read_with(addr - RAM, addr, source),
            ECHO..=ECHO_END => self.ram.read_with(addr - ECHO, addr, source),
            OAM..=OAM_END => self.oam.read_with(addr - OAM, addr, source),
            UN_1..=UN_1_END => self.un_1.read_with(addr - UN_1, addr, source),
            IO..=IO_END => self.io.read_with(addr - IO, addr, source),
            HRAM..=HRAM_END => self.hram.read_with(addr - HRAM, addr, source),
            END => self.ie.read()
        }
    }

    fn write_with(&mut self, addr: u16, value: u8, source: Source) {
        match addr {
            ROM..=ROM_END => self.rom.write_with(addr - ROM, value, addr, source),
            SROM..=SROM_END => self.srom.write_with(addr - SROM, value, addr, source),
            VRAM..=VRAM_END => self.vram.write_with(addr - VRAM, value, addr, source),
            SRAM..=SRAM_END => self.sram.write_with(addr - SRAM, value, addr, source),
            RAM..=RAM_END => self.ram.write_with(addr - RAM, value, addr, source),
            ECHO..=ECHO_END => self.ram.write_with(addr - ECHO, value, addr, source),
            OAM..=OAM_END => self.oam.write_with(addr - OAM, value, addr, source),
            UN_1..=UN_1_END => self.un_1.write_with(addr - UN_1, value, addr, source),
            IO..=IO_END => self.io.write_with(addr - IO, value, addr, source),
            HRAM..=HRAM_END => self.hram.write_with(addr - HRAM, value, addr, source),
            END => { self.ie.write_with(0, value, addr, source) }
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.write(addr, value);
    }

    fn lock(&mut self) {
        self.rom.lock(Source::Dma);
        self.srom.lock(Source::Dma);
        self.ram.lock(Source::Dma);
        self.sram.lock(Source::Dma);
        self.vram.lock(Source::Ppu);
        self.oam.lock(Source::Dma);
    }

    fn unlock(&mut self) {
        self.rom.unlock(Source::Dma);
        self.srom.unlock(Source::Dma);
        self.ram.unlock(Source::Dma);
        self.sram.unlock(Source::Dma);
        self.vram.unlock(Source::Ppu);
        self.oam.unlock(Source::Dma);
    }

    fn mbc(&self) -> Ref<dyn MBCController> {
        self.mbc.as_ref().borrow()
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
        self.last = None;
        self.status = match self.status {
            MemStatus::ReqRead(addr) => {
                let v = self.read(addr);
                self.last = Some(Op::Read(addr, v));
                MemStatus::Read(v)
            },
            MemStatus::ReqWrite(addr) => MemStatus::Write(addr),
            st => st
        };
    }

    /// Debug function
    /// will return a range starting from start and up to len bytes long, if possible.
    /// Will end early if the underlying memory range is smaller.
    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        match start {
            ROM..=ROM_END => self.rom.get_range(start, len),
            SROM..=SROM_END => self.srom.get_range(start, len),
            VRAM..=VRAM_END => self.vram.get_range(start, len),
            SRAM..=SRAM_END => self.sram.get_range(start, len),
            RAM..=RAM_END => self.ram.get_range(start, len),
            OAM..=OAM_END => self.oam.get_range(start, len),
            UN_1..=UN_1_END => self.un_1.get_range(start, len),
            IO..=IO_END => self.io.get_range(start, len),
            HRAM..=HRAM_END => self.hram.get_range(start, len),
            END => self.rom.get_range(start, len),
            ECHO..=ECHO_END => self.ram.get_range(start - ECHO + RAM, len)
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.last = Some(Op::Write(addr, value));
        self.write(addr, value);
    }

    fn direct_read(&self, offset: u16) -> u8 {
        self.read(offset)
    }
}
