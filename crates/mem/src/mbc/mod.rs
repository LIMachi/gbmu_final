use std::path::PathBuf;
use shared::{mem::*, rom::{Rom, Mbc as Mbcs}};
use shared::io::{IO, IODevice};
use crate::boot::Boot;

pub mod mbc0;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;
pub mod mbc5;

pub trait MemoryController {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self where Self: Sized;
    fn ram_dump(&self) -> Vec<u8>;

    fn rom_bank(&self) -> usize { 0 }
    fn ram_bank(&self) -> usize { 0 }
}

pub(crate) trait Mbc: MemoryController + Mem {
    fn is_boot(&self) -> bool { false }
    fn unmap(&mut self) -> Box<dyn Mbc> { unreachable!() }
    fn tick(&mut self) { }
}

pub struct Unplugged { }
impl Mem for Unplugged { }
impl Mbc for Unplugged { }

impl MemoryController for Unplugged {
    fn new(_rom: &Rom, _ram: Vec<u8>) -> Self where Self: Sized {
        Self { }
    }
    fn ram_dump(&self) -> Vec<u8> { vec![] }
}

pub struct Controller {
    sav: Option<PathBuf>,
    inner: Box<dyn Mbc>
}

impl Default for Controller {
    fn default() -> Self { Controller::unplugged() }
}

impl Controller {
    pub fn new(rom: &Rom, cgb: bool) -> Self {
        let (sav, ram) = if rom.header.cartridge.capabilities().save() {
            let sav = rom.location.clone().join(&rom.filename).with_extension("sav");
            let ram = if let Some(mut f) = std::fs::File::open(&sav).ok() {
                use std::io::Read;
                let mut v = Vec::with_capacity(rom.header.ram_size.size());
                f.read_to_end(&mut v).expect("failed to read save");
                v
            } else { vec![0xAF; rom.header.ram_size.size()] };
            (Some(sav), ram)
        } else { (None, vec![0xAF; rom.header.ram_size.size()]) };
        let inner: Box<dyn Mbc> = match rom.header.cartridge.mbc() {
            Mbcs::MBC0 => Box::new(Boot::<mbc0::Mbc0>::new(rom, ram, cgb)),
            Mbcs::MBC1 => Box::new(Boot::<mbc1::Mbc1>::new(rom, ram, cgb)),
            Mbcs::MBC2 => Box::new(Boot::<mbc2::Mbc2>::new(rom, ram, cgb)),
            Mbcs::MBC3 => Box::new(Boot::<mbc3::Mbc3>::new(rom, ram, cgb)),
            Mbcs::MBC5 => Box::new(Boot::<mbc5::Mbc5>::new(rom, ram, cgb)),
            Mbcs::Unknown => unimplemented!()
        };

        Self {
            sav,
            inner
        }
    }

    pub fn skip_boot(mut self) -> Self {
        self.post();
        self
    }

    pub fn unplugged() -> Self {
        Self { sav: None, inner: Box::new(Unplugged { }) }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let ram = self.inner.ram_dump();
        if ram.is_empty() { return }
        if let Some(sav) = &self.sav {
            log::info!("dumping ram to save file {:?}", sav);
            use std::io::Write;
            std::fs::File::create(&sav).ok()
                .map(|mut f| f.write_all(&ram).expect("failed to write savefile"));
        }
    }
}

impl MBCController for Controller {
    fn rom_bank(&self) -> usize { self.inner.rom_bank() }
    fn ram_bank(&self) -> usize { self.inner.ram_bank() }
    fn tick(&mut self) { self.inner.tick(); }

    fn post(&mut self) { self.inner = self.inner.unmap(); }
}

impl Mem for Controller {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.inner.read(addr, absolute)
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        self.inner.value(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.inner.write(addr, value, absolute);
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.inner.get_range(st, len)
    }

    fn lock(&mut self, access: Source) { self.inner.lock(access) }
    fn unlock(&mut self, access: Source) { self.inner.unlock(access) }
}

impl IODevice for Controller {
    fn write(&mut self, io: IO, _: u8, _: &mut dyn IOBus) {
        if io == IO::POST && self.inner.is_boot() {
            log::info!("--- POST ----");
            self.post();
        }
    }
}
