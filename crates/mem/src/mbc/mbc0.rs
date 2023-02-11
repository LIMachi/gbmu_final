use std::rc::Rc;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::PathBuf;
use shared::{mem::*, rom::Rom};

struct Mbc {
    sav: Option<PathBuf>,
    rom: Vec<u8>,
    ram: Vec<u8>
}

impl Mem for Mbc {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            ROM..=SROM_END => self.rom[absolute as usize],
            SRAM..=SRAM_END => self.ram[addr as usize],
            a => unreachable!("unexpected addr {a:#06X}")
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let s = st as usize;
        let end = s + len as usize;
        match st {
            ROM..=SROM_END => self.rom[s..end].to_vec(),
            SRAM..=SRAM_END => self.ram[s..end].to_vec(),
            _ => vec![]
        }
    }
}

impl Mbc {
    pub const ROM_SIZE: usize = 32768;

    pub fn new(rom: &Rom) -> Self {
        // TODO check battery flag in rom
        let (sav, ram) = if rom.header.cartridge.capabilities().save() {
            let sav = rom.location.clone().join(&rom.filename).with_extension("sav");
            let ram = if let Some(mut f) = std::fs::File::open(&sav).ok() {
                let mut v = Vec::with_capacity(rom.header.ram_size.size());
                f.read_to_end(&mut v).expect("failed to read save");
                v
            } else { vec![0xAF; rom.header.ram_size.size()] };
            (Some(sav), ram)
        } else { (None, vec![0xAF; rom.header.ram_size.size()]) };
        Self {
            sav,
            rom: rom.raw().clone(),
            ram
        }
    }
}

impl Drop for Mbc {
    fn drop(&mut self) {
        log::info!("dumping ram to save file");
        if self.ram.is_empty() { return }
        if let Some(sav) = &self.sav {
            std::fs::File::create(&sav).ok()
                .map(|mut f| f.write_all(&self.ram).expect("failed to write savefile"));
        }
    }
}

#[derive(Clone)]
pub struct Controller {
    inner: Rc<RefCell<Mbc>>
}

impl Controller {
    pub fn new(rom: &Rom) -> Self {
        Self { inner: Rc::new(RefCell::new(Mbc::new(rom))) }
    }
}

impl MBCController for Controller {
    fn rom(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
    fn srom(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
    fn sram(&self) -> Rc<RefCell<dyn Mem>> { self.inner.clone() }
}
