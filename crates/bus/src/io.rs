use shared::io::{IO, IOReg};
use shared::mem::Mem;

pub struct IORegs {
    range: Vec<IOReg>
}

impl IORegs {
    pub fn init() -> Self {
        Self {
            range: (0..128).into_iter().map(|_| IOReg::default()).collect()
        }
    }

    pub fn io(&self, io: IO) -> IOReg {
        self.range[io as u16 as usize - shared::mem::IO as usize].clone()
    }
}

impl Mem for IORegs {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        self.range.get(addr as usize).map(|x| x.read()).expect(format!("read outside of IOReg range {addr:#06X}").as_str())
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        self.range.get_mut(addr as usize).map(|mut x| x.write(value)).expect(format!("write outside of IOReg range {addr:#06X}").as_str());
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        let end = ((st + len) as usize).min(self.range.len());
        let st = (st as usize).min(self.range.len() - 1);
        self.range[st..end].iter().map(|x| x.value()).collect()
    }
}
