use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, OAM};

#[derive(Default)]
pub struct Dma {
    reg: IOReg,
    st: u16,
    p: usize,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            reg: Default::default(),
            st: 0,
            p: 160
        }
    }

    pub fn tick(&mut self, bus: &mut dyn IOBus) {
        if self.reg.dirty() {
            self.reg.reset_dirty();
            self.p = 0;
            self.st = (self.reg.value() as u16) << 8;
        }
        if self.p != 160 {
            let v = bus.read(self.st + self.p as u16);
            bus.write(OAM + self.p as u16, v);
            self.p += 1;
        }
    }
}

impl Device for Dma {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.reg = bus.io(IO::DMA);
    }
}
