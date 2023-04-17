use shared::io::{IO, IODevice};
use shared::mem::{IOBus, OAM, Source};

pub struct Dma {
    st: u16,
    p: usize
}

impl Default for Dma {
    fn default() -> Self {
        Self { st: 0, p: 160 }
    }
}

impl Dma {
    pub fn tick(&mut self, bus: &mut dyn IOBus) {
        if self.p != 160 {
            let v = bus.read_with(self.st + self.p as u16, Source::Dma);
            bus.write_with(OAM + self.p as u16, v, Source::Dma);
            self.p += 1;
            if self.p == 160 {
                bus.unlock();
            }
        }
    }
}

impl IODevice for Dma {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        if io == IO::DMA {
            self.p = 0;
            self.st = (v as u16) << 8;
            bus.lock();
        }
    }
}
