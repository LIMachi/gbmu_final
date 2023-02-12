use std::borrow::Borrow;
use shared::io::{IO, IOReg};
use shared::mem::*;

const BANK_SIZE: usize = 0x1000;
const WRAM_SIZE: usize = 2 * BANK_SIZE;

pub struct Wram {
    banks: Vec<Vec<u8>>,
    svbk: IOReg,
    cgb: bool
}

impl Mem for Wram {
    fn read(&self, addr: u16, _: u16) -> u8 {
        let (bank, addr) = match addr as usize {
            0..=BANK_SIZE => (0, addr as usize),
            BANK_SIZE..=WRAM_SIZE => (self.bank(), addr as usize - BANK_SIZE),
            _ => unreachable!()
        };
        self.banks[bank][addr]
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        let (bank, addr) = match addr as usize {
            0..=BANK_SIZE => (0, addr as usize),
            BANK_SIZE..=WRAM_SIZE => (self.bank(), addr as usize - BANK_SIZE),
            _ => unreachable!()
        };
        self.banks[bank][addr] = value;
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let (bank, st, end) = match st {
            RAM..=WRAM_HALF_END => { let st = (st - RAM) as usize; let end = (st + len as usize).min(BANK_SIZE); (0, st, end) },
            WRAM_HALF..=RAM_END => { let st = (st - WRAM_HALF) as usize; let end = (st + len as usize).min(BANK_SIZE); (self.bank(), st, end) },
            _ => unreachable!()
        };
        self.banks[bank][st..end].to_vec()
    }
}

impl Wram {
    pub fn new(cgb: bool) -> Self {
        let banks = if cgb { (0..8) } else { (0..2) }
            .into_iter().map(|_| vec![0; BANK_SIZE]).collect();
        Self {
            banks,
            cgb,
            svbk: IOReg::default()
        }
    }

    pub fn bank(&self) -> usize {
        match (self.cgb, self.svbk.read()) {
            (false, _) => 1,
            (true, v) => { if v != 0 { v as usize } else { 1 }}
        }
    }
}

impl IODevice for Wram {
    fn configure(mut self, bus: &dyn IOBus) -> Self {
        self.svbk = bus.io(IO::SVBK);
        self.svbk.direct_write(1);
        self
    }
}
