use std::cell::RefCell;
use std::rc::Rc;

pub type IOReg = Rc<RefCell<u8>>;

pub struct IoRegs {
    io: [Option<IOReg>; 128]
}

impl IoRegs {
    pub fn new() -> Self {
        Self { io: [None; 128] }
    }

    /// relative addr
    pub fn request(&mut self, addr: u16) -> IOReg {
        if self.io.get(addr).is_none() {
            self.io[addr] = Some(Rc::new(RefCell::new(0)));
        }
        self.io[addr].unwrap()
    }
}

pub trait IOBus {
    fn request(&self, addr: u16) -> IOReg;
}

pub trait RequestIO {
    fn configure(&mut self, bus: &mut dyn IOBus);
}

