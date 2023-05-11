use std::io::{Read, Write};
use std::path::PathBuf;
use dyn_clone::DynClone;
use serde::{Deserializer, Serializer};

use shared::{mem::*, rom::{Mbc as Mbcs, Rom}, rom};
use shared::serde::{Deserialize, Serialize};

use crate::boot::Boot;
use crate::mbc::mbc0::Mbc0;
use crate::mbc::mbc1::Mbc1;
use crate::mbc::mbc2::Mbc2;
use crate::mbc::mbc3::Mbc3;
use crate::mbc::mbc5::Mbc5;

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

pub(crate) trait Mbc: MemoryController + Mem + DynClone {
    fn is_boot(&self) -> bool { false }
    fn as_serde(&self) -> Option<MbcsEnum>;
    fn unmap(&mut self) -> Box<dyn Mbc> { unreachable!() }
    fn tick(&mut self) {}
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Unplugged {}

impl Mem for Unplugged {}

impl Mbc for Unplugged {
    fn as_serde(&self) -> Option<MbcsEnum> {
        None
    }
}

impl MemoryController for Unplugged {
    fn new(_rom: &Rom, _ram: Vec<u8>) -> Self where Self: Sized {
        Self {}
    }
    fn ram_dump(&self) -> Vec<u8> { vec![] }
}

#[derive(Serialize, Deserialize)]
pub struct Controller {
    header: rom::Header,
    sav: Option<PathBuf>,
    inner: Box<dyn Mbc>,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum MbcsEnum {
    BOOT(Boot),
    MBC0(Mbc0),
    MBC1(Mbc1),
    MBC2(Mbc2),
    MBC3(Mbc3),
    MBC5(Mbc5)
}

impl Serialize for Box<dyn Mbc> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.as_serde().serialize(serializer)
    }
}

impl <'de> Deserialize<'de> for Box<dyn Mbc> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Deserialize::deserialize(deserializer).map(|e: Option<MbcsEnum>| {
            match e {
                None => Box::new(Unplugged{}) as Box<dyn Mbc>,
                Some(MbcsEnum::BOOT(m)) => Box::new(m) as Box<dyn Mbc>,
                Some(MbcsEnum::MBC0(m)) => Box::new(m) as Box<dyn Mbc>,
                Some(MbcsEnum::MBC1(m)) => Box::new(m) as Box<dyn Mbc>,
                Some(MbcsEnum::MBC2(m)) => Box::new(m) as Box<dyn Mbc>,
                Some(MbcsEnum::MBC3(m)) => Box::new(m) as Box<dyn Mbc>,
                Some(MbcsEnum::MBC5(m)) => Box::new(m) as Box<dyn Mbc>,
            }
        })
    }
}

impl Clone for Controller {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            sav: self.sav.clone(),
            inner: dyn_clone::clone_box(&*self.inner)
        }
    }
}

impl Default for Controller {
    fn default() -> Self { Controller::unplugged() }
}

impl Controller {
    pub fn new(rom: &Rom, cgb: bool) -> Self {
        let (sav, ram) = if rom.header.cartridge.capabilities().save() {
            let sav = rom.location.clone().join(&rom.filename);
            log::info!("Trying to load save data...");
            let ram = if let Some(mut f) = std::fs::File::open(&sav.with_extension("sav")).ok() {
                let mut v = Vec::with_capacity(rom.header.ram_size.size());
                f.read_to_end(&mut v).expect("failed to read save");
                v
            } else {
                log::info!("No save detected, fresh file");
                vec![0xAF; rom.header.ram_size.size()]
            };
            (Some(sav), ram)
        } else {
            (None, vec![0xAF; rom.header.ram_size.size()])
        };
        let inner: Box<dyn Mbc> = match rom.header.cartridge.mbc() {
            Mbcs::MBC0 => Box::new(Boot::new(cgb, mbc0::Mbc0::new(rom, ram))),
            Mbcs::MBC1 => Box::new(Boot::new(cgb, mbc1::Mbc1::new(rom, ram))),
            Mbcs::MBC2 => Box::new(Boot::new(cgb, mbc2::Mbc2::new(rom, ram))),
            Mbcs::MBC3 => Box::new(Boot::new(cgb, mbc3::Mbc3::new(rom, ram))),
            Mbcs::MBC5 => Box::new(Boot::new(cgb, mbc5::Mbc5::new(rom, ram))),
            Mbcs::Unknown => unimplemented!()
        };

        Self {
            sav,
            inner,
            header: rom.header.clone(),
        }
    }

    pub fn skip_boot(mut self) -> Self {
        self.post();
        self
    }

    pub fn save(&self, autosave: bool) {
        let ram = self.inner.ram_dump();
        if ram.is_empty() { return; }
        if let Some(path) = &self.sav {
            use std::fs::File;
            let file = path.with_extension(if autosave { "autosav" } else { "sav" });
            log::info!("Saving... ({path:?})");
            let mut backup = vec![];
            File::open(&file).and_then(|mut x| x.read_to_end(&mut backup)).ok();
            File::create(path.with_extension("bak"))
                .and_then(|mut x| x.write_all(&backup))
                .unwrap_or_else(|e| log::warn!("Failed to save backup ({e:?})"));
            File::create(&file)
                .and_then(|mut x| x.write_all(&ram))
                .unwrap_or_else(|e| { log::warn!("save failed: {e:?}"); });
        }
    }

    pub fn unplugged() -> Self {
        Self { sav: None, header: rom::Header::default(), inner: Box::new(Unplugged {}) }
    }
}

impl MBCController for Controller {
    fn rom_bank(&self) -> usize { self.inner.rom_bank() }
    fn ram_bank(&self) -> usize { self.inner.ram_bank() }
    fn tick(&mut self) { self.inner.tick(); }

    fn post(&mut self) {
        if self.inner.is_boot() {
            log::info!("---- POST ----");
            self.inner = self.inner.unmap();
        }
    }
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

