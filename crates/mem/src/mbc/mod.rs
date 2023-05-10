use std::fmt::Formatter;
use std::io::{Read, Write};
use std::path::PathBuf;
use dyn_clone::DynClone;
use serde::ser::SerializeStruct;
use serde::{de, Deserializer, Serializer};
use serde::de::{MapAccess, SeqAccess, Visitor};

use shared::{mem::*, rom::{Mbc as Mbcs, Rom}, rom};
use shared::serde::{Deserialize, Serialize};
use crate::boot;

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

pub(crate) trait Mbc: MemoryController + Mem + DynClone {
    fn is_boot(&self) -> bool { false }
    fn kind(&self) -> u8 { 0 }
    fn raw(&self) -> Vec<u8> { vec![] }
    fn unmap(&mut self) -> Box<dyn Mbc> { unreachable!() }
    fn tick(&mut self) {}
}

#[derive(Default, Clone)]
pub struct Unplugged {}

impl Mem for Unplugged {}

impl Mbc for Unplugged {}

impl MemoryController for Unplugged {
    fn new(_rom: &Rom, _ram: Vec<u8>) -> Self where Self: Sized {
        Self {}
    }
    fn ram_dump(&self) -> Vec<u8> { vec![] }
}

pub struct Controller {
    header: rom::Header,
    sav: Option<PathBuf>,
    inner: Box<dyn Mbc>,
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

impl Serialize for Controller {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut s = serializer.serialize_struct("Controller", 3)?;
        s.serialize_field("sav", &self.sav)?;
        s.serialize_field("mbc", &self.inner.kind())?;
        s.serialize_field("raw", &self.inner.raw())?;
        s.end()
    }
}

impl <'de> Deserialize<'de> for Controller {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Sav, Mbc, Raw }

        struct ControllerVisitor;

        impl <'de> Visitor<'de> for ControllerVisitor {
            type Value = Controller;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("struct Controller")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                let sav: Option<PathBuf> = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let mbc: u8 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let raw: Vec<u8> = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let header = rom::Header::new(raw[..0x150].try_into().unwrap());
                let inner = match mbc {
                    0 => mbc0::Mbc0::from_raw(raw),
                    1 => mbc1::Mbc1::from_raw(raw),
                    2 => mbc2::Mbc2::from_raw(raw),
                    3 => mbc3::Mbc3::from_raw(raw),
                    5 => mbc5::Mbc5::from_raw(raw),
                    _ => Box::new(Unplugged::default())
                };
                Ok(Controller{header, sav, inner})
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                let mut sav: Option<Option<PathBuf>> = None;
                let mut mbc: Option<u8> = None;
                let mut raw: Option<Vec<u8>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Sav => {
                            if sav.is_some() {
                                return Err(de::Error::duplicate_field("sav"));
                            }
                            sav = Some(map.next_value()?);
                        },
                        Field::Mbc => {
                            if mbc.is_some() {
                                return Err(de::Error::duplicate_field("mbc"));
                            }
                            mbc = Some(map.next_value()?);
                        },
                        Field::Raw => {
                            if raw.is_some() {
                                return Err(de::Error::duplicate_field("raw"));
                            }
                            raw = Some(map.next_value()?);
                        }
                    }
                }
                let sav = sav.ok_or_else(|| de::Error::missing_field("sav"))?;
                let mbc = mbc.ok_or_else(|| de::Error::missing_field("mbc"))?;
                let raw = raw.ok_or_else(|| de::Error::missing_field("raw"))?;
                let header = rom::Header::new(raw[..0x150].try_into().unwrap());
                let inner = match mbc {
                    0 => mbc0::Mbc0::from_raw(raw),
                    1 => mbc1::Mbc1::from_raw(raw),
                    2 => mbc2::Mbc2::from_raw(raw),
                    3 => mbc3::Mbc3::from_raw(raw),
                    5 => mbc5::Mbc5::from_raw(raw),
                    _ => Box::new(Unplugged::default())
                };
                Ok(Controller{header, sav, inner})
            }
        }

        deserializer.deserialize_struct("Controller", &["sav", "mbc", "raw"], ControllerVisitor)
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
            Mbcs::MBC0 => Box::new(Boot::<mbc0::Mbc0>::new(rom, ram, cgb)),
            Mbcs::MBC1 => Box::new(Boot::<mbc1::Mbc1>::new(rom, ram, cgb)),
            Mbcs::MBC2 => Box::new(Boot::<mbc2::Mbc2>::new(rom, ram, cgb)),
            Mbcs::MBC3 => Box::new(Boot::<mbc3::Mbc3>::new(rom, ram, cgb)),
            Mbcs::MBC5 => Box::new(Boot::<mbc5::Mbc5>::new(rom, ram, cgb)),
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

