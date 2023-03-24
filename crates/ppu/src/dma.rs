use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, OAM, Source};


#[derive(Default)]
pub struct Dma {
    reg: IOReg,
    st: u16,
    p: usize
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
            bus.lock();
        }
        if self.p != 160 {
            let v = bus.read_with(self.st + self.p as u16, Source::Dma);
            // log::debug!("copy {:#06X} {:#04X} {:#06X}", self.st + self.p as u16, v, OAM + self.p as u16);
            bus.write_with(OAM + self.p as u16, v, Source::Dma);
            self.p += 1;
            if self.p == 160 {
                bus.unlock();
            }
        }
    }
}

impl Device for Dma {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.reg = bus.io(IO::DMA);
    }
}
