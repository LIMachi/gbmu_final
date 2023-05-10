use serde::{Deserialize, Serialize};
use shared::mem::*;
use shared::rom::Rom;
use shared::utils::rtc::Rtc;

use crate::mbc::{Mbc, MemoryController};

const BANK_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0x2000;

const RAM_ENABLE: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16 = 0x2000;
const ROM_BANK_END: u16 = 0x2FFF;
const ROM_BANK_H: u16 = 0x3000;
const ROM_BANK_H_END: u16 = 0x3FFF;
const RAM_BANK: u16 = 0x4000;
const RAM_BANK_END: u16 = 0x5FFF;

#[derive(Clone, Serialize, Deserialize)]
pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    enabled_ram: bool,
    rom_bank: usize,
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
}

impl Mbc5 {
    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn Mbc> {
        Box::new(bincode::deserialize::<Self>(raw.as_slice()).unwrap())
    }
}

impl Mem for Mbc5 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            ROM..=ROM_END => self.rom[addr as usize],
            SROM..=SROM_END => {
                let addr = addr as usize + self.rom_bank * BANK_SIZE;
                if addr > self.rom.len() { panic!("out of bounds cartridge rom read at {absolute}"); }
                self.rom[addr]
            }
            SRAM..=SRAM_END => {
                let addr = addr as usize + self.ram_bank * RAM_SIZE;
                if addr > self.ram.len() { panic!("out of bounds cartridge ram read at {absolute}"); }
                self.ram[addr]
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match absolute {
            RAM_ENABLE..=RAM_ENABLE_END => self.enabled_ram = (value & 0xF) == 0xA,
            ROM_BANK..=ROM_BANK_END => self.rom_bank = ((self.rom_bank & 0x100) | (value as usize)) & self.rom_banks,
            ROM_BANK_H..=ROM_BANK_H_END => self.rom_bank = (((value as usize & 0x1) << 8) | (self.rom_bank & 0xFF)) & self.rom_banks,
            RAM_BANK..=RAM_BANK_END => self.ram_bank = (value as usize & 0xF) & self.ram_banks,
            SRAM..=SRAM_END => {
                let addr = addr as usize + self.ram_bank as usize * RAM_SIZE;
                self.ram[addr] = value;
            }
            _ => {}
        }
    }

    fn get_range(&self, st: u16, _: u16) -> Vec<u8> {
        match st {
            ROM => self.rom[..BANK_SIZE].to_vec(),
            SROM => {
                let st = BANK_SIZE * self.rom_bank;
                self.rom[st..(st + BANK_SIZE)].to_vec()
            }
            SRAM => {
                let st = RAM_SIZE * self.ram_bank;
                self.rom[st..(st + RAM_SIZE)].to_vec()
            }
            _ => vec![]
        }
    }
}

impl MemoryController for Mbc5 {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self {
        Self {
            rom: rom.raw(),
            ram,
            enabled_ram: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks: rom.header.rom_size.mask(),
            ram_banks: rom.header.ram_size.mask(),
        }
    }

    fn ram_dump(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn rom_bank(&self) -> usize { self.rom_bank }
    fn ram_bank(&self) -> usize {
        self.ram_bank
    }
}

impl super::Mbc for Mbc5 {
    fn kind(&self) -> u8 {
        5
    }

    fn raw(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}
