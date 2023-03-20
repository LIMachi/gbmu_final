use std::cell::RefCell;
use std::rc::Rc;
use crate::io::{IO, IOReg};

pub mod lock;
pub use lock::*;

pub trait Mem {
    fn read(&self, _addr: u16, _absolute: u16) -> u8 {
        0xFF
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        self.read(addr, absolute)
    }

    fn write(&mut self, _addr: u16, _value: u8, _absolute: u16) { }

    fn get_range(&self, _st: u16, _len: u16) -> Vec<u8> { vec![] }

    fn read_with(&self, addr: u16, absolute: u16, _access: lock::Source) -> u8 {
        self.read(addr, absolute)
    }

    fn write_with(&mut self, addr: u16, value: u8, absolute: u16, _access: lock::Source) {
        self.write(addr, value, absolute)
    }

    fn lock(&mut self, _access: Source) { }
    fn unlock(&mut self, _access: Source) { }
}

impl Mem for Rc<RefCell<dyn Mem>> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().read(addr, absolute)
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().value(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.as_ref().borrow_mut().write(addr, value, absolute)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.as_ref().borrow().get_range(st, len)
    }

    fn read_with(&self, addr: u16, absolute: u16, access: Source) -> u8 {
        self.as_ref().borrow().read_with(addr, absolute, access)
    }

    fn write_with(&mut self, addr: u16, value: u8, absolute: u16, access: Source) {
        self.as_ref().borrow_mut().write_with(addr, value, absolute, access)
    }

    fn lock(&mut self, access: Source) { self.as_ref().borrow_mut().lock(access); }
    fn unlock(&mut self, access: Source) { self.as_ref().borrow_mut().unlock(access); }
}

impl<T: Mem> Mem for Rc<RefCell<T>> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().read(addr, absolute)
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().value(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.as_ref().borrow_mut().write(addr, value, absolute)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.as_ref().borrow().get_range(st, len)
    }

    fn read_with(&self, addr: u16, absolute: u16, access: Source) -> u8 {
        self.as_ref().borrow().read_with(addr, absolute, access)
    }

    fn write_with(&mut self, addr: u16, value: u8, absolute: u16, access: Source) {
        self.as_ref().borrow_mut().write_with(addr, value, absolute, access)
    }
    fn lock(&mut self, access: Source) { self.as_ref().borrow_mut().lock(access); }
    fn unlock(&mut self, access: Source) { self.as_ref().borrow_mut().unlock(access); }
}


pub trait MemoryBus {
    fn with_mbc<C: MBCController>(self, controller: &mut C) -> Self;
    fn with_ppu<P: PPU>(self, ppu: &mut P) -> Self;
    fn with_wram<R: Device + Mem + 'static>(self, ram: R) -> Self;
}

pub trait IOBus {
    fn configure<Dev: 'static + Device>(self, dev: &mut Dev) -> Self where Self: 'static + Sized { dev.configure(&self); self }
    fn io(&self, io: IO) -> IOReg;
    fn read(&self, addr: u16) -> u8;
    fn is_cgb(&self) -> bool;

    fn read_with(&self, addr: u16, source: Source) -> u8;
    fn write_with(&mut self, addr: u16, value: u8, source: Source);

    fn write(&mut self, addr: u16, value: u8);

    /// DMA memory access lock
    fn lock(&mut self);
    fn unlock(&mut self);
}

pub trait Device {
    fn configure(&mut self, _bus: &dyn IOBus) { }
}

impl Device for () { }


pub trait PPU: Device {
    fn vram(&self) -> Rc<RefCell<dyn Mem>>;
    fn oam(&self) -> Rc<RefCell<dyn Mem>>;
}

pub trait MBCController: Device {
    fn rom(&self) -> Rc<RefCell<dyn Mem>>;
    fn srom(&self) -> Rc<RefCell<dyn Mem>>;
    fn sram(&self) -> Rc<RefCell<dyn Mem>>;
}

pub const ROM: u16 = 0x0;
pub const ROM_END: u16 = 0x3FFF;
pub const SROM: u16 = 0x4000;
pub const SROM_END: u16 = 0x7FFF;
pub const VRAM: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const SRAM: u16 = 0xA000;
pub const SRAM_END: u16 = 0xBFFF;
pub const RAM: u16 = 0xC000;
pub const WRAM_HALF_END: u16 = 0xCFFF;
pub const WRAM_HALF: u16 = 0xD000;
pub const RAM_END: u16 = 0xDFFF;
pub const ECHO: u16 = 0xE000;
pub const ECHO_END: u16 = 0xFDFF;
pub const OAM: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const UN_1: u16 = 0xFEA0;
pub const UN_1_END: u16 = 0xFEFF;
pub const IO: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const HRAM: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const END: u16 = 0xFFFF;
