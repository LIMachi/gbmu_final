use shared::mem::*;
use shared::rom::Rom;
use super::Mbc;

const BANK_SIZE: usize = 0x4000;
const RAM_SIZE : usize = 0x0200;

pub struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    rom_banks: usize,
    ram_enabled: bool
}

impl super::Mem for Mbc2 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        match absolute {
            ROM..=ROM_END => self.rom[addr as usize],
            SROM..=SROM_END => {
                let bank = self.rom_bank % self.rom_banks;
                let addr = addr as usize + bank * BANK_SIZE;
                if addr > self.rom.len() {
                    eprintln!("out of bounds cartridge rom read at {absolute}");
                    0xFF
                } else { self.rom[addr] }
            },
            SRAM..=SRAM_END => {
                if self.ram_enabled {
                    let addr = addr as usize & 0x1FF;
                    self.ram[addr] & 0xF
                }
                else { 0xFF }  //TODO je sais pas 0xF ou 0xFF
            }
            _ => unreachable!()
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        match absolute {
            ROM..=ROM_END => if absolute & 0x100 == 0 {
                self.ram_enabled = value == 0xA;
            } else {
                let bank = value & 0xF;
                self.rom_bank = if bank == 0 { 1 } else { bank as usize };
            },
            SRAM..=SRAM_END => {
                if self.ram_enabled {
                    let addr = addr as usize & 0x1FF;
                    self.ram[addr] = value & 0xF;
                }
            }
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
                    if st != SRAM { return vec![] };
                    std::iter::once(self.ram.clone())
                        .cycle()
                        .take(len as usize / 512)
                        .chain(std::iter::once(self.ram[0..(len as usize) % 512].to_vec()))
                        .flatten()
                        .collect()
                }
            _ => vec![]
        }
    }
}

impl super::MemoryController for Mbc2 {
    fn new(rom: &Rom, mut ram: Vec<u8>) -> Self where Self: Sized {
        if ram.is_empty() { ram = vec![0xA; RAM_SIZE]; }
        Self {
            rom: rom.raw(),
            ram,
            rom_bank: 1,
            rom_banks:rom.header.rom_size.banks(),
            ram_enabled: false
        }
    }

    fn ram_dump(&self) -> Vec<u8> { self.ram.clone() }

    fn rom_bank(&self) -> usize { self.rom_bank }
}

impl Device for Mbc2 { }
impl Mbc for Mbc2 { }
