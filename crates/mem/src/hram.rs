use shared::mem::{HRAM, Mem};

const STACK_SIZE: usize = 0x7F;

pub struct Hram {
    mem: Vec<u8>
}

impl Mem for Hram {
    fn read(&self, addr: u16, _absolute: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8, _absolute: u16) {
        self.mem[addr as usize] = value;
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let st = st as usize - HRAM as usize;
        let end = st + len as usize;
        self.mem[st..end.min(STACK_SIZE)].to_vec()
    }
}

impl Hram {
    pub fn new() -> Self {
        Hram {
            mem: vec![0; STACK_SIZE]
        }
    }
}
