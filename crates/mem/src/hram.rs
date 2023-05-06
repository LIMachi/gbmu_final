use serde::{Deserialize, Serialize};
use shared::mem::Mem;

const STACK_SIZE: usize = 0x7F;

#[derive(Serialize, Deserialize)]
pub struct Hram {
    mem: Vec<u8>,
}

impl Mem for Hram {
    fn read(&self, addr: u16, _absolute: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8, _absolute: u16) {
        self.mem[addr as usize] = value;
    }

    fn get_range(&self, _st: u16, _len: u16) -> Vec<u8> {
        self.mem.clone()
    }
}

impl Hram {
    pub fn new() -> Self {
        Hram {
            mem: vec![0; STACK_SIZE]
        }
    }
}
