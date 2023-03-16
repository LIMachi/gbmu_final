use shared::mem::{Device, IOBus, Mem};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Source {
    Ppu = 0x0,
    Hdma = 0x1,
    Dma = 0x2
}

pub struct Lock<M: Mem, O: Ord + Eq + Copy = Source> {
    inner: M,
    lock: Vec<O>
}

pub trait Locked {
    fn lock(self) -> Lock<Self> where Self: Sized + Mem;
}

impl<M: Mem> Locked for M {
    fn lock(self) -> Lock<Self> { Lock::new(self) }
}

impl<M: Mem, O: Ord + Eq + Copy> Lock<M, O> {
    fn new(inner: M) -> Self {
        Self { inner, lock: vec![] }
    }

    /*fn lock(&mut self, level: O) {
        if self.lock.iter().find(|&&x| x < level).is_none() {
            self.lock.push(level);
        }
    }
    fn unlock(&mut self, level: O) {
        self.lock.drain_filter(|x| *x == level);
    }*/


    pub fn get<F: Fn(&M) -> u8>(&self, source: O, f: F) -> u8 {
        match source {
            v if self.lock.iter().find(|x| x > &&v).is_none() => f(&self.inner),
            _ => 0xFF
        }
    }

    pub fn get_mut(&mut self, source: Option<O>) -> Option<&mut M> {
        match source {
            None => Some(&mut self.inner),
            Some(v) if self.lock.iter().find(|x| x > &&v).is_none() => Some(&mut self.inner),
            _ => None
        }
    }

    pub fn inner(&self) -> &M { &self.inner }
    pub fn inner_mut(&mut self) -> &mut M { &mut self.inner }
}

impl<M: Mem, O: Ord + Copy> Mem for Lock<M, O> {
    fn read(&self, addr: u16, absolute: u16) -> u8 {
        if self.lock.is_empty() { self.inner.read(addr, absolute) } else { 0xFF }
    }

    fn value(&self, addr: u16, absolute: u16) -> u8 {
        self.inner.value(addr, absolute)
    }

    fn write(&mut self, addr: u16, value: u8, absolute: u16) {
        if self.lock.is_empty() { self.inner.write(addr, value, absolute); }
    }

    fn get_range(&self, st: u16, len: u16) -> Vec<u8> {
        self.inner.get_range(st, len)
    }
}

impl<M: Mem + Device, O: Ord + Copy> Device for Lock<M, O> {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.inner.configure(bus);
    }
}
