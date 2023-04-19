use shared::mem::*;
use shared::rom::Rom;
use shared::utils::rtc::Rtc;

use crate::mbc::MemoryController;

const BANK_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0x2000;

const RAM_ENABLE: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16 = 0x2000;
const ROM_BANK_END: u16 = 0x3FFF;
const RAM_BANK: u16 = 0x4000;
const RAM_BANK_END: u16 = 0x5FFF;
const LATCH: u16 = 0x6000;
const LATCH_END: u16 = 0x7FFF;

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    enabled_ram: bool,
    rom_bank: usize,
    ram_bank: usize,
    rom_banks: usize,
    ram_banks: usize,
    rtc: Rtc,
    latch: bool,
}

impl Mem for Mbc3 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            ROM..=ROM_END => self.rom[addr as usize],
            SROM..=SROM_END => {
                let bank = self.rom_bank % self.rom_banks;
                let addr = addr as usize + bank * BANK_SIZE;
                if addr >= self.rom.len() {
                    eprintln!("out of bounds cartridge rom read at {absolute}");
                    0xFF
                } else { self.rom[addr] }
            }
            SRAM..=SRAM_END => {
                match self.ram_bank {
                    n @ 0x8..=0xC => self.rtc.read(n as u8),
                    n => {
                        let addr = addr as usize + n * RAM_SIZE;
                        if addr >= self.ram.len() {
                            eprintln!("out of bounds cartridge ram read at {absolute}");
                            0xFF
                        } else { self.ram[addr] }
                    }
                }
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match absolute {
            RAM_ENABLE..=RAM_ENABLE_END => self.enabled_ram = (value & 0xF) == 0xA,
            ROM_BANK..=ROM_BANK_END => {
                let bank = value as usize % self.rom_banks;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            RAM_BANK..=RAM_BANK_END => self.ram_bank = value as usize & 0xF,
            LATCH..=LATCH_END => {
                let old = self.latch;
                self.latch = value != 0;
                if !old && self.latch { self.rtc.latch(); }
            }
            SRAM..=SRAM_END => {
                match self.ram_bank {
                    n @ 0x8..=0xC => self.rtc.write(n as u8, value),
                    n => {
                        let addr = addr as usize + n as usize * RAM_SIZE;
                        self.ram[addr] = value;
                    }
                }
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

impl MemoryController for Mbc3 {
    fn new(rom: &Rom, mut ram: Vec<u8>) -> Self {
        let raw = ram.split_off(rom.header.ram_size.size());
        let rtc = Rtc::deserialize(raw).unwrap_or_default();
        Self {
            rom: rom.raw(),
            ram,
            rtc,
            enabled_ram: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banks: rom.header.rom_size.banks(),
            ram_banks: rom.header.ram_size.banks(),
            latch: true,
        }
    }

    fn ram_dump(&self) -> Vec<u8> {
        let mut dump = self.ram.clone();
        dump.append(&mut self.rtc.serialize());
        dump
    }

    fn rom_bank(&self) -> usize { self.rom_bank }
    fn ram_bank(&self) -> usize {
        self.ram_bank
    }
}

impl super::Mbc for Mbc3 {
    fn tick(&mut self) {
        self.rtc.tick();
    }
}

