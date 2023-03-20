use std::cell::{RefCell};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;
use super::{Mem, Device, IOBus};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Source {
    Hdma = 0x0,
    Ppu = 0x1,
    Dma = 0x2
}

pub struct Lock<M: ?Sized + Mem> {
    inner: Box<M>,
    lock: HashSet<Source>
}

pub trait Locked {
    fn locked(self) -> Lock<Self> where Self: Mem;
    fn locked_cell(self) -> Lock<Rc<RefCell<dyn Mem>>> where Self: 'static;
    // fn cell_locked(self) -> Rc<RefCell<Lock<dyn Mem>>> where Self: 'static;
}

impl<M: Mem> Locked for M {
    fn locked(self) -> Lock<Self> { Lock::new(self) }
    fn locked_cell(self) -> Lock<Rc<RefCell<dyn Mem>>> where M: 'static {
        let cell = RefCell::new(self);
        Lock::new(Rc::new(cell))
    }
    // fn cell_locked(self) -> Rc<RefCell<Lock<dyn Mem>>> where M: 'static {
    //     Rc::new(RefCell::new(Lock::new(self)))
    // }
}

impl<M: Mem> Lock<M> {
    pub fn new(inner: M) -> Self {
        Self { inner: Box::new(inner), lock: HashSet::with_capacity(4) }
    }

    pub fn lock(&mut self, level: Source) {
       self.lock.insert(level);
    }
    pub fn unlock(&mut self, level: Source) {
        self.lock.remove(&level);
    }

    pub fn get<F: Fn(&M) -> u8>(&self, source: Source, f: F) -> u8 {
        match source {
            v if self.lock.iter().find(|x| x > &&v).is_none() => f(&self.inner),
            _ => 0xFF
        }
    }

    pub fn get_mut(&mut self, source: Source) -> Option<&mut M> {
        match source {
            v if self.lock.iter().find(|x| x > &&v).is_none() => Some(&mut self.inner),
            _ => None
        }
    }

    pub fn inner(&self) -> &M { &self.inner }
    pub fn inner_mut(&mut self) -> &mut M { &mut self.inner }
}

impl<M: Mem> Mem for Lock<M> {
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

    fn read_with(&self, addr: u16, absolute: u16, access: Source) -> u8 {
        self.get(access, |inner| inner.read_with(addr, absolute, access))
    }

    fn write_with(&mut self, addr: u16, value: u8, absolute: u16, access: Source) {
        self.get_mut(access).map(|a| a.write_with(addr, value, absolute, access));
    }

    fn lock(&mut self, access: Source) { self.lock(access); }
    fn unlock(&mut self, access: Source) { self.unlock(access); }
}

impl<M: Mem + Device> Device for Lock<M> {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.inner.configure(bus);
    }
}