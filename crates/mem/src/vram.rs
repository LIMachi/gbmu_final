use std::collections::HashSet;

use shared::mem::Mem;

const BANK_SIZE: usize = 0x2000;

enum Storage {
    DMG([u8; BANK_SIZE]),
    CGB([[u8; BANK_SIZE]; 2], usize),
}

impl Storage {
    fn read_bank(&self, addr: u16, bank: usize) -> u8 {
        match self {
            Storage::DMG(_) if bank == 1 => 0,
            Storage::DMG(bank) => bank[addr as usize],
            Storage::CGB(banks, _) => banks[bank][addr as usize]
        }
    }
}

impl Mem for Storage {
    fn read(&self, addr: u16, _absolute: u16) -> u8 {
        use Storage::*;
        match self {
            DMG(mem) => mem[addr as usize],
            CGB(mem, bank) => mem[*bank][addr as usize]
        }
    }

    fn write(&mut self, addr: u16, value: u8, _absolute: u16) {
        use Storage::*;
        match self {
            DMG(mem) => mem[addr as usize] = value,
            CGB(mem, bank) => mem[*bank][addr as usize] = value
        }
    }

    fn get_range(&self, _st: u16, _len: u16) -> Vec<u8> {
        use Storage::*;
        match self {
            DMG(mem) => mem[..].to_vec(),
            CGB(mem, _) => [&mem[0][..], &mem[1][..]].concat(),
        }
    }
}

pub struct Vram {
    pub tile_cache: HashSet<usize>,
    mem: Storage,
}

impl Vram {
    pub fn tile_data(&self, tile: usize, bank: usize) -> [u8; 64] {
        let mut out = [0; 64];
        for y in 0..8 {
            let low = self.mem.read_bank((tile * 16 + y * 2) as u16, bank);
            let high = self.mem.read_bank((tile * 16 + y * 2 + 1) as u16, bank);
            for x in 0..8 {
                let num = ((low >> x) & 1) | (((high >> x) & 1) << 1);
                out[7 - x + y * 8] = num;
            }
        }
        out
    }
}

impl Mem for Vram {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.mem.read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        let bank = match &self.mem {
            Storage::CGB(_, 1) => 384usize,
            _ => 0
        };
        if addr < 0x1800 { self.tile_cache.insert(addr as usize / 16 + bank); }
        self.mem.write(addr, value, absolute);
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.mem.get_range(st, len)
    }
}

impl Vram {
    pub fn new(cgb: bool) -> Self {
        Self {
            tile_cache: HashSet::with_capacity(768),
            mem: if cgb {
                Storage::CGB([[0; BANK_SIZE]; 2], 0)
            } else {
                Storage::DMG([0; BANK_SIZE])
            },
        }
    }

    pub fn read_bank(&self, addr: u16, bank: usize) -> u8 {
        self.mem.read_bank(addr, bank)
    }

    pub fn switch_bank(&mut self, new: u8) {
        if let Storage::CGB(_, bank) = &mut self.mem {
            *bank = (new & 1) as usize;
        }
    }
}
