use dyn_clone::DynClone;
use shared::mem::Mem;
use shared::rom::Rom;
use crate::mbc::{Mbc, MemoryController};

#[derive(Clone)]
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

#[derive(Clone)]
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

trait BootSection: Mem + DynClone {
    fn new() -> Self where Self: Sized;
    fn contains(&self, addr: u16) -> bool;
}

pub(crate) struct Boot<MBC: Mbc> {
    boot: Box<dyn BootSection>,
    inner: Option<MBC>
}

impl <MBC: Mbc> Clone for Boot<MBC> {
    fn clone(&self) -> Self {
        Self {
            boot: dyn_clone::clone_box(&*self.boot),
            inner: if let Some(i) = &self.inner { Some(dyn_clone::clone(i)) } else { None }
        }
    }
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
            self.boot.value(addr, absolute)
        } else {
            self.inner.as_ref().unwrap().value(addr, absolute)
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
    fn new(_: &Rom, _: Vec<u8>) -> Self where Self: Sized {
        unreachable!()
    }

    fn ram_dump(&self) -> Vec<u8> { self.inner.as_ref().unwrap().ram_dump() }
    fn rom_bank(&self) -> usize { 0 }
    fn ram_bank(&self) -> usize { 0 }
}

impl<MBC: Mbc> Boot<MBC> {
    pub fn new(rom: &Rom, ram: Vec<u8>, cgb: bool) -> Self {
        Self {
            boot: if cgb { Box::new(CgbBoot::new())} else { Box::new(DmgBoot::new()) },
            inner: Some(MBC::new(rom, ram))
        }
    }
}
