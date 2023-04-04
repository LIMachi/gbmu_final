use shared::mem::{Device, IOBus, Mem};
use shared::rom::Rom;
use crate::mbc::{Mbc, MemoryController};

struct DmgBoot {
    raw: Vec<u8>
}

impl Mem for DmgBoot {
    fn read(&self, _addr: u16, absolute: u16) -> u8 {
        self.raw[absolute as usize]
    }
    fn value(&self, _addr: u16, absolute: u16) -> u8 {
        self.raw[absolute as usize]
    }
}

impl BootSection for DmgBoot {
    fn new() -> Self where Self: Sized {
        Self { raw: include_bytes!("../../../assets/boot/dmg_boot.bin").to_vec() }
    }
    fn contains(&self, addr: u16) -> bool { (0..0x100).contains(&addr) }
}

struct CgbBoot {
    raw: Vec<u8>
}

impl BootSection for () {
    fn new() -> Self where Self: Sized { () }
    fn contains(&self, _addr: u16) -> bool { false }
}

impl BootSection for CgbBoot {
    fn new() -> Self where Self: Sized {
        Self { raw: include_bytes!("../../../assets/boot/cgb_boot.bin").to_vec() }
    }
    fn contains(&self, addr: u16) -> bool {
        (0..0x100).contains(&addr) || (0x200..=0x08FF).contains(&addr)
    }
}

impl Mem for CgbBoot {
    fn read(&self, _addr: u16, absolute: u16) -> u8 {
        self.raw[absolute as usize]
    }

    fn value(&self, _addr: u16, absolute: u16) -> u8 {
        self.raw[absolute as usize]
    }
}

trait BootSection: Mem {
    fn new() -> Self where Self: Sized;
    fn contains(&self, addr: u16) -> bool;
}

pub(crate) struct Boot<MBC: Mbc> {
    boot: Box<dyn BootSection>,
    inner: Option<MBC>
}

impl<MBC: Mbc> Mem for Boot<MBC> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        if self.boot.contains(absolute) {
            self.boot.read(addr, absolute)
        } else {
            self.inner.as_ref().unwrap().read(addr, absolute)
        }
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        if self.boot.contains(absolute) {
            self.boot.read(addr, absolute)
        } else {
            self.inner.as_ref().unwrap().read(addr, absolute)
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.inner.as_ref().unwrap().get_range(st, len)
    }
}

impl<M: Mbc + 'static> Mbc for Boot<M> {
    fn is_boot(&self) -> bool { true }
    fn unmap(&mut self) -> Box<dyn Mbc> { Box::new(self.inner.take().unwrap()) }
}

impl<MBC: Mbc> MemoryController for Boot<MBC> {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self where Self: Sized {
        Self {
            boot: Box::new(()),
            inner: Some(MBC::new(rom, ram))
        }
    }

    fn ram_dump(&self) -> Vec<u8> { self.inner.as_ref().unwrap().ram_dump() }
    fn rom_bank(&self) -> usize { 0 }
    fn ram_bank(&self) -> usize { 0 }
}

impl<MBC: Mbc> Device for Boot<MBC> {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.inner.as_mut().unwrap().configure(bus);
        self.boot = if bus.is_cgb() {
            Box::new(CgbBoot::new())
        } else {
            Box::new(DmgBoot::new())
        };
    }
}
