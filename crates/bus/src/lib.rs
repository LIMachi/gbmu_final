#![feature(trait_upcasting)]

use mem::{Hram, Oam, Vram, Wram};
use shared::{cpu::MemStatus, cpu::Op, mem::*};
use shared::io::{IO, IOReg, IORegs};

mod timer;
mod devices;

pub use timer::Timer;
pub use devices::Devices;

pub use devices::Settings;

pub struct Empty {}
impl Mem for Empty {}

pub struct Bus {
    clock: u8,
    mbc: Lock<mem::mbc::Controller>,
    ram: Lock<Wram>,
    hram: Hram,
    un_1: Empty,
    vram: Lock<Vram>,
    oam: Lock<Oam>,
    io: IORegs,
    ie: IOReg,
    status: MemStatus,
    last: Option<Op>
}

impl Bus {
    pub fn new(cgb: bool) -> Self {
        Self {
            clock: 0,
            mbc: mem::mbc::Controller::unplugged().locked(),
            last: None,
            io: IORegs::init(cgb),
            vram: Vram::new(cgb).locked(),
            oam: Oam::new().locked(),
            ram: Wram::new(cgb).locked(),
            hram: Hram::new(),
            un_1: Empty {},
            ie: IOReg::with_access(IO::IE.access()),
            status: MemStatus::ReqRead(0x0)
        }
    }

    pub fn with_mbc(mut self, mut controller: mem::mbc::Controller) -> Self {
        self.mbc = controller.locked();
        self
    }

    pub fn skip_boot(mut self, skip: bool, console: u8) -> Self {
        if skip {
            self.status = MemStatus::ReqRead(0x100);
            self.io.skip_boot(console);
        }
        self
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            ROM..=ROM_END => self.mbc.read(addr, addr),
            SROM..=SROM_END => self.mbc.read(addr - SROM, addr),
            VRAM..=VRAM_END => self.vram.read(addr - VRAM, addr),
            SRAM..=SRAM_END => self.mbc.read(addr - SRAM, addr),
            RAM..=RAM_END => self.ram.read(addr - RAM, addr),
            ECHO..=ECHO_END => self.ram.read(addr - ECHO, addr),
            OAM..=OAM_END => self.oam.read(addr - OAM, addr),
            UN_1..=UN_1_END => self.un_1.read(addr - UN_1, addr),
            IO..=IO_END => self.io.read(addr - IO, addr),
            HRAM..=HRAM_END => self.hram.read(addr - HRAM, addr),
            END => self.ie.read()
        }
    }

    pub fn last(&mut self) -> Option<Op> {
        self.last.take()
    }

    pub fn tick(&mut self, devices: &mut Devices, clock: u8, settings: Settings) -> bool {
        if self.clock == 127 {
            self.mbc.inner_mut().tick();
            self.clock = 0;
        } else { self.clock += 1; }
        devices.joy.tick(&mut self.io);

        let ds = self.io.io(IO::KEY1).bit(7) != 0;
        if clock == 0 || clock == 2 {
            let tick = devices.hdma.tick(self);
            if clock == 0 || ds {
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
                devices.serial.tick(&mut self.io);
                devices.timer.tick(&mut self.io);
                devices.dma.tick(self);
                if !tick {
                    devices.cpu.cycle(self);
                    if let Some(Op::Write(addr, v)) = self.last {
                        if matches!(addr, IO..=IO_END) {
                            devices.io_write(addr, v, self);
                        }
                    }
                }
            }
        }
        devices.ppu.tick(&mut self.io, &mut self.oam, &mut self.vram, &mut devices.lcd);
        devices.apu.tick(&mut self.io, ds, settings.sound);
        let bp = settings.breakpoints.tick(&devices.cpu, self.last());
        devices.cpu.reset_finished();
        bp
    }
}

impl shared::cpu::Bus for Bus {
    fn status(&self) -> MemStatus {
        self.status
    }

    fn update(&mut self, status: MemStatus) {
        self.status = status;
    }
    /// Debug function
    /// will return a range starting from start and up to len bytes long, if possible.
    /// Will end early if the underlying memory range is smaller.
    fn get_range(&self, start: u16, len: u16) -> Vec<u8> {
        match start {
            ROM..=ROM_END => self.mbc.get_range(start, len),
            SROM..=SROM_END => self.mbc.get_range(start, len),
            VRAM..=VRAM_END => self.vram.get_range(start, len),
            SRAM..=SRAM_END => self.mbc.get_range(start, len),
            RAM..=RAM_END => self.ram.get_range(start, len),
            OAM..=OAM_END => self.oam.get_range(start, len),
            UN_1..=UN_1_END => self.un_1.get_range(start, len),
            IO..=IO_END => self.io.get_range(start, len),
            HRAM..=HRAM_END => self.hram.get_range(start, len),
            END => vec![self.ie.value()],
            ECHO..=ECHO_END => self.ram.get_range(start - ECHO + RAM, len)
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.last = Some(Op::Write(addr, value));
        match addr {
            ROM..=ROM_END => self.mbc.write(addr - ROM, value, addr),
            SROM..=SROM_END => self.mbc.write(addr - SROM, value, addr),
            VRAM..=VRAM_END => self.vram.write(addr - VRAM, value, addr),
            SRAM..=SRAM_END => self.mbc.write(addr - SRAM, value, addr),
            RAM..=RAM_END => self.ram.write(addr - RAM, value, addr),
            ECHO..=ECHO_END => self.ram.write(addr - ECHO, value, addr),
            OAM..=OAM_END => self.oam.write(addr - OAM, value, addr),
            UN_1..=UN_1_END => self.un_1.write(addr - UN_1, value, addr),
            IO..=IO_END => {
                self.io.write(addr - IO, value, addr);
                match addr {
                    0xFF50 if self.io.writable(IO::POST) => {
                        self.io.post();
                    },
                    0xFF4C if self.io.writable(IO::KEY0) => {
                        self.io.compat_mode();
                    },
                    0xFF70 if self.io.writable(IO::SVBK) => {
                        self.ram.inner_mut().switch_bank(value);
                    },
                    0xFF4F if self.io.writable(IO::VBK) => {
                        self.vram.inner_mut().switch_bank(value);
                    },
                    _ => {}
                }
            },
            HRAM..=HRAM_END => self.hram.write(addr - HRAM, value, addr),
            END => { self.ie.write(0, value, addr) }
        };
    }

    fn direct_read(&self, offset: u16) -> u8 {
        self.read(offset)
    }

    fn int_reset(&mut self, bit: u8) {
        self.io.io_mut(IO::IF).reset(bit);
    }

    fn int_set(&mut self, bit: u8) {
        self.io.int_set(bit);
    }

    fn interrupt(&mut self) -> u8 {
        (self.io.io(IO::IF).read() & self.ie.read()) & 0x1F
    }
}

impl IOBus for Bus {
    fn io_mut(&mut self, io: IO) -> &mut IOReg {
        match io {
            IO::IE => &mut self.ie,
            io => self.io.io_mut(io)
        }
    }

    fn io(&self, io: IO) -> &IOReg {
        match io {
            IO::IE => &self.ie,
            io => self.io.io(io)
        }
    }

    fn io_addr(&mut self, io: u16) -> Option<&mut IOReg> {
        self.io.io_addr(io)
    }

    fn io_regs(&mut self) -> &mut IORegs { &mut self.io }

    fn read(&self, addr: u16) -> u8 {
        self.read(addr)
    }

    fn is_cgb(&self) -> bool { self.io.io(IO::CGB).value() != 0 }

    fn read_with(&self, addr: u16, source: Source) -> u8 {
        match addr {
            ROM..=ROM_END => self.mbc.read_with(addr - ROM, addr, source),
            SROM..=SROM_END => self.mbc.read_with(addr - SROM, addr, source),
            VRAM..=VRAM_END => self.vram.read_with(addr - VRAM, addr, source),
            SRAM..=SRAM_END => self.mbc.read_with(addr - SRAM, addr, source),
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
            VRAM..=VRAM_END => self.vram.write_with(addr - VRAM, value, addr, source),
            OAM..=OAM_END => self.oam.write_with(addr - OAM, value, addr, source),
            _ => unreachable!()
        }
    }

    fn lock(&mut self) {
        self.mbc.lock(Source::Dma);
        self.ram.lock(Source::Dma);
        self.vram.lock(Source::Ppu);
        self.oam.lock(Source::Dma);
    }

    fn unlock(&mut self) {
        self.mbc.unlock(Source::Dma);
        self.ram.unlock(Source::Dma);
        self.vram.unlock(Source::Ppu);
        self.oam.unlock(Source::Dma);
    }

    fn mbc(&self) -> Box<&dyn MBCController> {
        Box::new(self.mbc.inner())
    }
}

impl ppu::VramAccess for Bus {
    fn vram(&self) -> &Vram {
        self.vram.inner()
    }

    fn vram_mut(&mut self) -> &mut Vram {
        self.vram.inner_mut()
    }

    fn oam(&self) -> &Oam {
        self.oam.inner()
    }

    fn oam_mut(&mut self) -> &mut Oam {
        self.oam.inner_mut()
    }
}
