use shared::mem::*;
use shared::rom::Rom;
use crate::mbc::MemoryController;

const BANK_SIZE: usize = 0x4000;
const RAM_SIZE : usize = 0x2000;

const RAM_ENABLE: u16     = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16       = 0x2000;
const ROM_BANK_END: u16   = 0x2FFF;
const ROM_BANK_H: u16     = 0x3000;
const ROM_BANK_H_END: u16 = 0x3FFF;
const RAM_BANK: u16       = 0x4000;
const RAM_BANK_END: u16   = 0x5FFF;

pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    enabled_ram: bool,
    rom_bank: usize,
    ram_bank: u8
}

impl Mem for Mbc5 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            ROM..=ROM_END => self.rom[addr as usize],
            SROM..=SROM_END => {
                let addr = addr as usize + self.rom_bank * BANK_SIZE;
                if addr > self.rom.len() { panic!("out of bounds cartridge rom read at {absolute}"); }
                self.rom[addr]
            },
            SRAM..=SRAM_END => {
                let addr = addr as usize + self.ram_bank as usize * RAM_SIZE;
                if addr > self.ram.len() { panic!("out of bounds cartridge ram read at {absolute}"); }
                self.ram[addr]
            },
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match absolute {
            RAM_ENABLE..=RAM_ENABLE_END => self.enabled_ram = (value & 0xF) == 0xA,
            ROM_BANK..=ROM_BANK_END => self.rom_bank = (self.rom_bank & 0x100) | value as usize,
            ROM_BANK_H..=ROM_BANK_H_END => self.rom_bank = ((value as usize & 0x1) << 8) | (self.rom_bank & 0xFF),
            RAM_BANK..=RAM_BANK_END => self.ram_bank = value & 0xF,
            SRAM..=SRAM_END => {
                let addr = addr as usize + self.ram_bank as usize * BANK_SIZE;
                if addr > self.ram.len() { panic!("out of bounds cartridge ram write at [{}] {addr:#06X} {absolute:#06X}", self.ram_bank); }
                self.ram[addr] = value;
            },
            _ => unreachable!("mbc not supposed to write at {addr:#06X} [{absolute:#06X}]")
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let s = st as usize;
        match st {
            ROM..=ROM_END => self.rom[s..((st + len) as usize).min(BANK_SIZE)].to_vec(),
            SROM..=SROM_END => {
                let s = s - SROM as usize;
                let end = (s + len as usize).min(BANK_SIZE) + self.rom_bank * BANK_SIZE;
                let st = s + self.rom_bank * BANK_SIZE;
                self.rom[st..end].to_vec()
            },
            SRAM..=SRAM_END => {
                let s = s - SRAM as usize;
                let st = s + RAM_SIZE * self.ram_bank as usize;
                let end = (st + len as usize).min((self.ram_bank + 1) as usize * RAM_SIZE);
                self.ram[st..end].to_vec()
            },
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
            ram_bank: 0
        }
    }

    fn ram_dump(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn rom_bank_low(&self) -> u8 {
        (self.rom_bank & 0xFF) as u8
    }

    fn rom_bank_high(&self) -> u8 {
        (self.rom_bank >> 8) as u8
    }

    fn ram_bank(&self) -> u8 {
        self.ram_bank
    }
}
