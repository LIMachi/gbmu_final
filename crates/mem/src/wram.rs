use shared::mem::*;

const BANK_SIZE: usize = 0x1000;
const WRAM_SIZE: usize = 2 * BANK_SIZE;

enum Storage {
    Dmg([u8; WRAM_SIZE]),
    Cgb(usize, [[u8; BANK_SIZE]; 8])
}

impl Mem for Storage {
    fn read(&self, addr: u16, _: u16) -> u8 {
        let addr = addr as usize;
        match self {
            Storage::Dmg(v) => v[addr],
            Storage::Cgb(bank, banks) => {
                match addr as usize {
                    0..BANK_SIZE => banks[0][addr],
                    BANK_SIZE..WRAM_SIZE => banks[*bank][addr - BANK_SIZE],
                    _ => unreachable!()
                }
            }
        }
    }

    fn write(&mut self, addr: u16, value: u8, _: u16) {
        let addr = addr as usize;
        match self {
            Storage::Dmg(v) => v[addr] = value,
            Storage::Cgb(bank, banks) => {
                match addr as usize {
                    0..BANK_SIZE => banks[0][addr] = value,
                    BANK_SIZE..WRAM_SIZE => banks[*bank][addr - BANK_SIZE] = value,
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
            Storage::Cgb(bank, banks) => {
                match (st, len) {
                    (st @ BANK_SIZE..WRAM_SIZE, len) => banks[*bank][(st - BANK_SIZE)..(st + len - BANK_SIZE).min(BANK_SIZE)].to_vec(),
                    (st @ 0..BANK_SIZE, len) if st + len < BANK_SIZE => banks[0][st..(st + len)].to_vec(),
                    (st @ 0..BANK_SIZE, len) => {
                        let mut ret = banks[0][st..].to_vec();
                        ret.extend_from_slice(&banks[*bank][..(len - BANK_SIZE - st).min(BANK_SIZE)]);
                        ret
                    },
                    _ => unreachable!()
                }
            }
        }
    }
}

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
            storage: if cgb { Storage::Cgb(1, [[0; BANK_SIZE]; 8]) } else { Storage::Dmg([0; WRAM_SIZE]) }
        }
    }

    pub fn switch_bank(&mut self, new: u8) {
        if let Storage::Cgb(bank, _) = &mut self.storage {
            *bank = new.clamp(1, 7) as usize;
        }
    }
}
