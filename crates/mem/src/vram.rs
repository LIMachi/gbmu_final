use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, IODevice, Mem};

const BANK_SIZE: u16 = 0x1000;

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
}

pub struct Vram {
    mem: Storage,
    bank: IOReg,
}

impl Mem for Vram {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        let addr = addr + if self.mem.cgb() { (self.bank.read() & 0x1) as u16 } else { 0 } * BANK_SIZE;
        self.mem.read(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        let addr = addr + if self.mem.cgb() { (self.bank.read() & 0x1) as u16 } else { 0 } * BANK_SIZE;
        self.mem.write(addr, value, absolute);

    }
}

impl Vram {
    pub fn new(cgb: bool) -> Self {
        Self {
            mem: if cgb { Storage::CGB([0; BANK_SIZE as usize * 2]) } else { Storage::DMG([0; BANK_SIZE as usize]) },
            bank: IOReg::default()
        }
    }
}

impl IODevice for Vram {
    fn configure(mut self, bus: &dyn IOBus) -> Self {
        self.bank = bus.io(IO::VBK);
        self
    }
}
