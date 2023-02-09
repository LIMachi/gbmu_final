use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use log::warn;

pub trait Mem {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        warn!("read ignored on {absolute:#04X}: address is not mapped");
        0xFF
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        warn!("write ignored on {absolute:#04X}: address is read only");
    }
}

impl<T: Mem> Mem for Rc<RefCell<T>> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.as_ref().borrow_mut().write(addr, value, absolute)
    }
}

pub trait MemoryBus {
    fn set_rom(&mut self, rom: Rc<RefCell<dyn Mem>>);
    fn set_srom(&mut self, rom: Rc<RefCell<dyn Mem>>);
    fn set_sram(&mut self, rom: Rc<RefCell<dyn Mem>>);
}

pub trait MemoryController {
    fn register(&self, bus: &mut impl MemoryBus);
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
