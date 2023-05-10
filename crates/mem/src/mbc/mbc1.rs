use shared::mem;
use shared::mem::{Mem, SRAM, SROM_END};
use shared::rom::Rom;
use shared::utils::ToBox;

use crate::mbc::{Mbc, MemoryController};

const RAM_ENABLE: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16 = 0x2000;
const ROM_BANK_END: u16 = 0x3FFF;
const RAM_BANK: u16 = 0x4000;
const RAM_BANK_END: u16 = 0x5FFF;
const BANK_MODE: u16 = 0x6000;
const BANK_MODE_END: u16 = 0x7FFF;

#[derive(Clone)]
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

impl Mbc1 {
    pub(crate) fn from_raw(raw: Vec<u8>) -> Box<dyn Mbc> {
        let sl = std::mem::size_of::<usize>();
        let rom_banks = usize::from_le_bytes(raw[..sl].try_into().unwrap());
        let ram_banks = usize::from_le_bytes(raw[sl..2 * sl].try_into().unwrap());
        let ram_enable = raw[2 * sl] == 1;
        let rom_reg_1 = raw[2 * sl + 1];
        let rom_reg_2 = raw[2 * sl + 2];
        let bank_mode = raw[2 * sl + 3] == 1;
        let rom_end = 2 * sl + 4 + 0x4000 * rom_banks;
        let rom = raw[2 * sl + 4 .. rom_end].to_vec();
        let ram = raw[rom_end..].to_vec();
        Box::new(Self { rom_banks, rom, ram_banks, ram, ram_enable, rom_reg_1, rom_reg_2, bank_mode})
    }
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

impl super::Mbc for Mbc1 {
    fn kind(&self) -> u8 {
        1
    }

    fn raw(&self) -> Vec<u8> {
        let mut out = self.rom_banks.to_le_bytes().to_vec();
        out.extend(&self.ram_banks.to_le_bytes());
        out.push(if self.ram_enable { 1 } else { 0 });
        out.push(self.rom_reg_1);
        out.push(self.rom_reg_2);
        out.push(if self.bank_mode { 1 } else { 0 });
        out.extend(&self.rom);
        out.extend(&self.ram);
        out
    }
}
