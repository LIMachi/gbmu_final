use shared::mem::Mem;
use shared::rom::Rom;
use crate::mbc::MemoryController;

const RAM_ENABLE: u16     = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;
const ROM_BANK: u16       = 0x2000;
const ROM_BANK_END: u16   = 0x3FFF;
const RAM_BANK: u16       = 0x4000;
const RAM_BANK_END: u16   = 0x5FFF;
const BANK_MODE: u16      = 0x6000;
const BANK_MODE_END: u16  = 0x7FFF;

pub struct Mbc1 {
    ram_enable: bool,
    rom_bank: u8,
    bank_mode: bool,
    rom: Vec<u8>,
    ram: Vec<u8>
}

impl Mem for Mbc1 {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        todo!()
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        todo!()
    }
}

impl MemoryController for Mbc1 {
    fn new(rom: &Rom, ram: Vec<u8>) -> Self where Self: Sized {
        todo!()
    }

    fn ram_dump(&self) -> Vec<u8> {
        todo!()
    }

    fn ram_bank(&self) -> u8 {
        todo!()
    }

    fn rom_bank_high(&self) -> u8 {
        todo!()
    }

    fn rom_bank_low(&self) -> u8 {
        todo!()
    }
}
