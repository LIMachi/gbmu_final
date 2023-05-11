use shared::mem::Mem;
use shared::rom::Rom;
use shared::serde::{Deserialize, Serialize};

use crate::mbc::{Mbc, MemoryController, Unplugged};

struct DmgBoot {
    raw: Vec<u8>,
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

#[derive(Serialize, Deserialize)]
struct CgbBoot {
    raw: Vec<u8>,
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

pub(crate) struct Boot {
    boot: Box<dyn BootSection>,
    inner: Box<dyn Mbc>,
}


impl Mem for Boot {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        if self.boot.contains(absolute) {
            self.boot.read(addr, absolute)
        } else {
            self.inner.read(addr, absolute)
        }
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        if self.boot.contains(absolute) {
            self.boot.value(addr, absolute)
        } else {
            self.inner.value(addr, absolute)
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.inner.get_range(st, len)
    }
}

impl Mbc for Boot {
    fn is_boot(&self) -> bool { true }
    fn unmap(&mut self) -> Box<dyn Mbc> { std::mem::replace(&mut self.inner, Box::new(Unplugged {})) }
}

impl MemoryController for Boot {
    fn new(_: &Rom, _: Vec<u8>) -> Self where Self: Sized {
        unreachable!()
    }

    fn ram_dump(&self) -> Vec<u8> { self.inner.ram_dump() }
    fn rom_bank(&self) -> usize { 0 }
    fn ram_bank(&self) -> usize { 0 }
}

impl Boot {
    pub fn new<MBC: Mbc + 'static>(cgb: bool, mbc: MBC) -> Self {
        Self {
            boot: if cgb { Box::new(CgbBoot::new()) } else { Box::new(DmgBoot::new()) },
            inner: Box::new(mbc),
        }
    }
}
