use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus, OAM, Source};


#[derive(Default)]
pub struct Dma {
    reg: IOReg,
    st: u16,
    p: usize,
    running: bool,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            reg: Default::default(),
            st: 0,
            p: 160,
            running: false
        }
    }

    pub fn tick(&mut self, bus: &mut dyn IOBus) {
        if self.reg.dirty() {
            self.reg.reset_dirty();
            self.p = 0;
            self.running = true;
            self.st = (self.reg.value() as u16) << 8;
            bus.lock();
        }
        if !self.running { return ; }
        if self.p != 160 {
            let v = bus.read_with(self.st + self.p as u16, Source::Dma);
            bus.write_with(OAM + self.p as u16, v, Source::Dma);
            self.p += 1;
        } else {
            bus.unlock();
            self.running = false;
        }
    }
}

impl Device for Dma {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.reg = bus.io(IO::DMA);
    }
}
