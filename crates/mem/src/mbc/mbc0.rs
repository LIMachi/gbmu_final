use std::io::{Read, Write};
use std::path::PathBuf;
use shared::{mem::*, rom::Rom};

pub struct Mbc0 {
    rom: Vec<u8>,
    ram: Vec<u8>
}

impl Mem for Mbc0 {
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
            SRAM..=SRAM_END => self.ram[s.min(self.ram.len())..end.min(self.ram.len())].to_vec(),
            _ => vec![]
        }
    }
}

impl super::MemoryController for Mbc0 {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self {
        Self {
            rom: rom.raw().clone(),
            ram
        }
    }

    fn ram_dump(&self) -> Vec<u8> {
        vec![]
    }
}
