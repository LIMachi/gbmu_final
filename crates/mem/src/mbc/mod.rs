use std::{rc::Rc, cell::RefCell};
use std::path::PathBuf;
use shared::{mem::*, rom::{Rom, Mbc as Mbcs}};
use shared::utils::Cell;

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

trait Mbc: MemoryController + Mem {
}

impl<M: MemoryController + Mem> Mbc for M { }

pub struct Unplugged { }

impl MemoryController for Unplugged {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self where Self: Sized {
        Self { }
    }
    fn ram_dump(&self) -> Vec<u8> { vec![] }
}

impl Mem for Unplugged { }

#[derive(Clone)]
pub struct Controller {
    sav: Option<PathBuf>,
    inner: Rc<RefCell<dyn Mbc>>
}

impl Controller {
    pub fn new(rom: &Rom) -> Self {
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
        let inner: Rc<RefCell<dyn Mbc>> = match rom.header.cartridge.mbc() {
            Mbcs::MBC0 => mbc0::Mbc0::new(rom, ram).cell(),
            Mbcs::MBC1 => mbc1::Mbc1::new(rom, ram).cell(),
            Mbcs::MBC2 => unimplemented!(),
            Mbcs::MBC3 => unimplemented!(),
            Mbcs::MBC5 => mbc5::Mbc5::new(rom, ram).cell(),
            Mbcs::Unknown => unimplemented!()
        };

        Self {
            sav,
            inner
        }
    }

    pub fn unplugged() -> Self {
        Self { sav: None, inner: Unplugged { }.cell() }
    }

    pub fn rom_bank(&self) -> usize { self.inner.borrow().rom_bank() }
    pub fn ram_bank(&self) -> usize { self.inner.borrow().ram_bank() }
}

impl Drop for Controller {
    fn drop(&mut self) {
        let ram = self.inner.as_ref().borrow_mut().ram_dump();
        if ram.is_empty() { return }
        log::info!("dumping ram to save file");
        if let Some(sav) = &self.sav {
            use std::io::Write;
            std::fs::File::create(&sav).ok()
                .map(|mut f| f.write_all(&ram).expect("failed to write savefile"));
        }
    }
}

impl MBCController for Controller {
    fn rom(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
    fn srom(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
    fn sram(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
}

impl Device for Controller { }
