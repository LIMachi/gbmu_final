use serde::{Deserialize, Serialize};
use shared::mem::*;
use shared::utils::serde_arrays;

const BANK_SIZE: usize = 0x1000;

const INCLUSIVE_BANK_SIZE: usize = BANK_SIZE - 1;

const WRAM_SIZE: usize = 2 * BANK_SIZE;

const INCLUSIVE_WRAM_SIZE: usize = WRAM_SIZE - 1;

#[derive(Serialize, Deserialize, Clone)]
struct CgbStorage {
    #[serde(with = "serde_arrays")]
    bank0: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank1: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank2: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank3: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank4: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank5: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank6: [u8; BANK_SIZE],
    #[serde(with = "serde_arrays")]
    bank7: [u8; BANK_SIZE],
    selected: usize
}

impl CgbStorage {
    fn new() -> Self {
        Self {
            bank0: [0u8; BANK_SIZE],
            bank1: [0u8; BANK_SIZE],
            bank2: [0u8; BANK_SIZE],
            bank3: [0u8; BANK_SIZE],
            bank4: [0u8; BANK_SIZE],
            bank5: [0u8; BANK_SIZE],
            bank6: [0u8; BANK_SIZE],
            bank7: [0u8; BANK_SIZE],
            selected: 1
        }
    }

    fn bank(&self, selected: usize) -> &[u8; BANK_SIZE] {
        match selected {
            0 => &self.bank0,
            1 => &self.bank1,
            2 => &self.bank2,
            3 => &self.bank3,
            4 => &self.bank4,
            5 => &self.bank5,
            6 => &self.bank6,
            7 => &self.bank7,
            _ => unreachable!()
        }
    }

    fn mut_bank(&mut self, selected: usize) -> &mut [u8; BANK_SIZE] {
        match selected {
            0 => &mut self.bank0,
            1 => &mut self.bank1,
            2 => &mut self.bank2,
            3 => &mut self.bank3,
            4 => &mut self.bank4,
            5 => &mut self.bank5,
            6 => &mut self.bank6,
            7 => &mut self.bank7,
            _ => unreachable!()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
enum Storage {
    #[serde(with = "serde_arrays")]
    Dmg([u8; WRAM_SIZE]),
    Cgb(CgbStorage)
}

impl Mem for Storage {
    fn read(&self, addr: u16, _: u16) -> u8 {
        let addr = addr as usize;
        match self {
            Storage::Dmg(v) => v[addr],
            Storage::Cgb(c) => {
                match addr as usize {
                    0..=INCLUSIVE_BANK_SIZE => c.bank(0)[addr],
                    BANK_SIZE..=INCLUSIVE_WRAM_SIZE => c.bank(c.selected)[addr - BANK_SIZE],
                    _ => unreachable!()
                }
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8, _: u16) {
        let addr = addr as usize;
        match self {
            Storage::Dmg(v) => v[addr] = value,
            Storage::Cgb(c) => {
                match addr as usize {
                    0..=INCLUSIVE_BANK_SIZE => c.mut_bank(0)[addr] = value,
                    BANK_SIZE..=INCLUSIVE_WRAM_SIZE => c.mut_bank(c.selected)[addr - BANK_SIZE] = value,
                    _ => unreachable!()
                }
            }
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let st = (st - RAM) as usize;
        let len = len as usize;
        match self {
            Storage::Dmg(v) => v[st..(st + len).min(WRAM_SIZE)].to_vec(),
            Storage::Cgb(c) => {
                match (st, len) {
                    (st @ BANK_SIZE..=INCLUSIVE_WRAM_SIZE, len) => c.bank(c.selected)[(st - BANK_SIZE)..(st + len - BANK_SIZE).min(BANK_SIZE)].to_vec(),
                    (st @ 0..=INCLUSIVE_BANK_SIZE, len) if st + len < BANK_SIZE => c.bank(0)[st..(st + len)].to_vec(),
                    (st @ 0..=INCLUSIVE_BANK_SIZE, len) => {
                        let mut ret = c.bank(0)[st..].to_vec();
                        ret.extend_from_slice(&c.bank(c.selected)[..(len - BANK_SIZE - st).min(BANK_SIZE)]);
                        ret
                    },
                    _ => unreachable!()
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Wram {
    storage: Storage
}

impl Mem for Wram {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.storage.read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.storage.write(addr, value, absolute);
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.storage.get_range(st, len)
    }
}

impl Wram {
    pub fn new(cgb: bool) -> Self {
        Self {
            storage: if cgb { Storage::Cgb(CgbStorage::new()) } else { Storage::Dmg([0; WRAM_SIZE]) }
        }
    }

    pub fn switch_bank(&mut self, new: u8) {
        if let Storage::Cgb(c) = &mut self.storage {
            c.selected = new.clamp(1, 7) as usize;
        }
    }
}
