use shared::io::IO;
use shared::mem::{Device, IOBus, OAM, Source};

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
        let reg = bus.io(IO::DMA);
        if reg.dirty() {
            reg.reset_dirty();
            self.p = 0;
            self.st = (reg.value() as u16) << 8;
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

impl Device for Dma {}
