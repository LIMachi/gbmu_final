use shared::mem;
use shared::mem::Mem;
use shared::rom::Rom;

use crate::mbc::MemoryController;

const RAM_ENABLE: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16 = 0x2000;
const ROM_BANK_END: u16 = 0x3FFF;
const RAM_BANK: u16 = 0x4000;
const RAM_BANK_END: u16 = 0x5FFF;
const BANK_MODE: u16 = 0x6000;
const BANK_MODE_END: u16 = 0x7FFF;

pub struct Mbc1 {
    ram_banks: usize,
    rom_banks: usize,
    ram_enable: bool,
    rom_reg_1: u8,
    rom_reg_2: u8,
    bank_mode: bool,
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Mem for Mbc1 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        use mem::*;
        match absolute {
            ROM..=ROM_END => {
                let bank = if self.bank_mode { (self.rom_reg_2 as usize) << 5 } else { 0 } & self.rom_banks;
                self.rom[addr as usize | (bank << 14)]
            }
            SROM..=SROM_END => {
                let mut bank = self.rom_reg_1 as usize & self.rom_banks;
                if self.rom_reg_1 == 0 { bank = 1; }
                bank |= (self.rom_reg_2 as usize) << 5;
                bank &= self.rom_banks;
                self.rom[addr as usize | (bank << 14)]
            }
            SRAM..=SRAM_END => {
                if self.ram_enable {
                    let bank = if self.bank_mode { self.rom_reg_2 as usize & self.ram_banks } else { 0 };
                    self.ram[addr as usize | bank << 13]
                } else {
                    0xFF
                }
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        use mem::*;
        match absolute {
            RAM_ENABLE..=RAM_ENABLE_END => self.ram_enable = (value & 0xF) == 0xA,
            ROM_BANK..=ROM_BANK_END => self.rom_reg_1 = value & 0x1F,
            RAM_BANK..=RAM_BANK_END => self.rom_reg_2 = value & 0x3,
            BANK_MODE..=BANK_MODE_END => self.bank_mode = value != 0,
            SRAM..=SRAM_END => {
                if self.ram_enable {
                    let bank = if self.bank_mode { self.rom_reg_2 as usize & self.ram_banks } else { 0 };
                    let addr = addr as usize | (bank << 13);
                    self.ram[addr] = value;
                }
            }
            _ => {}
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        use mem::*;
        match st {
            ROM => self.rom[0..0x4000].to_vec(),
            SROM => {
                let mut bank = self.rom_reg_1 as usize & self.rom_banks;
                if self.rom_reg_1 == 0 { bank = 1; }
                bank |= (self.rom_reg_2 as usize) << 5;
                bank &= self.rom_banks;
                let mut st = bank << 14;
                let end = st + 0x4000;
                if end > self.rom.len() {
                    vec![0; 0x4000]
                } else {
                    self.rom[st..(st + 0x4000)].to_vec()
                }
            }
            SRAM => {
                let bank = if self.bank_mode { self.rom_reg_2 as usize & self.ram_banks } else { 0 };
                let st = bank << 13;
                let end = st + 0x2000;
                if end > self.ram.len() {
                    vec![0; 0x2000]
                } else {
                    self.ram[st..end].to_vec()
                }
            }
            _ => vec![]
        }
    }
}

impl MemoryController for Mbc1 {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self where Self: Sized {
        Self {
            ram_banks: rom.header.ram_size.mask(),
            rom_banks: rom.header.rom_size.mask(),
            ram_enable: false,
            rom_reg_1: 0,
            rom_reg_2: 0,
            bank_mode: false,
            rom: rom.raw(),
            ram,
        }
    }

    fn ram_dump(&self) -> Vec<u8> {
        self.ram.clone()
    }

    fn rom_bank(&self) -> usize {
        let mut bank = self.rom_reg_1 as usize & self.rom_banks;
        if self.rom_reg_1 == 0 { bank = 1; }
        bank |= (self.rom_reg_2 as usize) << 5;
        bank & self.rom_banks
    }

    fn ram_bank(&self) -> usize { if self.bank_mode { self.rom_reg_2 as usize & self.ram_banks } else { 0 } }
}

impl super::Mbc for Mbc1 {}
