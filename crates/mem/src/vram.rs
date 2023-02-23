use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, Mem};

const BANK_SIZE: u16 = 0x2000;

enum Storage {
    DMG([u8; BANK_SIZE as usize]),
    CGB([u8; 2 * BANK_SIZE as usize])
}

impl Storage {
    fn cgb(&self) -> bool {
        match self {
            Storage::DMG(_) => false,
            Storage::CGB(_) => true
        }
    }

    fn read_bank(&self, addr: u16, bank: usize) -> u8 {
        match self {
            Storage::DMG(bank) => bank[addr as usize],
            Storage::CGB(banks) => banks[addr as usize + (bank & 0x1) * BANK_SIZE as usize]
        }
    }
}

impl Mem for Storage {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        use Storage::*;
        match self {
            DMG(mem) => mem[addr as usize],
            CGB(mem) => mem[addr as usize]
        }
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        use Storage::*;
        match self {
            DMG(mem) => mem[addr as usize] = value,
            CGB(mem) => mem[addr as usize] = value
        }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        use Storage::*;
        match self {
            DMG(mem) => mem[..].to_vec(),
            CGB(mem) => mem[..].to_vec(),
        }
    }
}

pub struct Vram {
    mem: Storage,
    bank: IOReg,
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
        let addr = addr + if self.mem.cgb() { (self.bank.read() & 0x1) as u16 * BANK_SIZE } else { 0 };
        self.mem.read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        let addr = addr + if self.mem.cgb() { (self.bank.read() & 0x1) as u16 * BANK_SIZE } else { 0 };
        self.mem.write(addr, value, absolute);
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.mem.get_range(st, len)
    }
}

impl Vram {
    pub fn new(cgb: bool) -> Self {
        Self {
            mem: if cgb { Storage::CGB([0; BANK_SIZE as usize * 2]) } else { Storage::DMG([0; BANK_SIZE as usize]) },
            bank: IOReg::unset()
        }
    }

    pub fn read_bank(&self, addr: u16, bank: usize) -> u8 {
        self.mem.read_bank(addr, bank)
    }
}

impl Device for Vram {
    fn configure(&mut self, bus: &dyn IOBus) {
        if self.mem.cgb() {
            self.bank = bus.io(IO::VBK);
        }
    }
}
