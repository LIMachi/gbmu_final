use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use log::warn;
use crate::io::{IO, IOReg};

pub trait Mem {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        warn!("read ignored on {absolute:#04X}: address is not mapped");
        0xFF
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        warn!("write ignored on {absolute:#04X}: address is read only");
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> { vec![] }
}

impl<T: Mem> Mem for Rc<RefCell<T>> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.as_ref().borrow().read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.as_ref().borrow_mut().write(addr, value, absolute)
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.as_ref().borrow().get_range(st, len)
    }
}
//
// impl Mem for Box<dyn Mem + 'static> {
//     fn read(&self, addr: u16, absolute: u16) -> u8 {
//         self.as_ref().read(addr, absolute)
//     }
//
//     fn write(&mut self, addr: u16, value: u8, absolute: u16) {
//         self.as_mut().write(addr, value, absolute)
//     }
//
//     fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
//         self.as_ref().get_range(st, len)
//     }
// }

pub trait IODevice {
    fn configure(self, bus: &dyn IOBus) -> Self;
}

pub trait MemoryBus {
    fn with_mbc<C: MBCController>(self, controller: &mut C) -> Self;
    fn with_ppu<P: PPU>(self, ppu: &mut P) -> Self;
    fn with_wram<R: IODevice + Mem + 'static>(self, ram: R) -> Self;
    fn with_vram<R: IODevice + Mem + 'static>(self, ram: R) -> Self;
}

pub trait IOBus {
    fn io(&self, io: IO) -> IOReg;
}

pub trait Device {
    fn configure(&mut self, bus: &dyn IOBus) { }
}

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
