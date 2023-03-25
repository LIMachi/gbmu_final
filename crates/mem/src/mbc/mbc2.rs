use shared::mem::Device;
use shared::rom::Rom;
use super::Mbc;

pub struct Mbc2 { }
impl super::MemoryController for Mbc2 {
    fn new(_rom: &Rom, _ram: Vec<u8>) -> Self where Self: Sized {
        Self { }
    }

    fn ram_dump(&self) -> Vec<u8> { vec![] }
}
impl super::Mem for Mbc2 { }
impl Device for Mbc2 { }
impl Mbc for Mbc2 { }
