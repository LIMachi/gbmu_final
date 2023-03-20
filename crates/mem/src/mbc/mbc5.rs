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
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
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
                let addr = addr as usize + self.ram_bank * RAM_SIZE;
                if addr > self.ram.len() { panic!("out of bounds cartridge ram read at {absolute}"); }
                self.ram[addr]
            },
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match absolute {
            RAM_ENABLE..=RAM_ENABLE_END => self.enabled_ram = (value & 0xF) == 0xA,
            ROM_BANK..=ROM_BANK_END => self.rom_bank = ((self.rom_bank & 0x100) | (value as usize)) % self.rom_banks,
            ROM_BANK_H..=ROM_BANK_H_END => self.rom_bank = ((value as usize & 0x1) << 8) | (self.rom_bank & 0xFF),
            RAM_BANK..=RAM_BANK_END => self.ram_bank = (value as usize & 0xF) % self.ram_banks,
            SRAM..=SRAM_END => {
                let addr = addr as usize + self.ram_bank as usize * RAM_SIZE;
                self.ram[addr] = value;
            },
            _ => {}
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
                if end > self.ram.len() {
                    eprintln!("hope you're not reading from that rambank ({}) because it's not mapped !", self.ram_bank);
                    return vec![];
                }
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
            ram_bank: 0,
            rom_banks: rom.header.rom_size.banks(),
            ram_banks: rom.header.ram_size.banks()
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

impl Device for Mbc5 { }
impl super::Mbc for Mbc5 { }
