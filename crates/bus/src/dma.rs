use shared::cpu::Bus;
use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};

#[derive(Default)]
pub struct Dma {
    reg: IOReg
}

impl Dma {
    pub fn tick(&mut self, bus: &mut dyn Bus) {
        if self.reg.dirty() {
            log::info!("start DMA routine");
            self.reg.reset_dirty();
            // TODO trigger dma routine.
        }
    }
}

impl Device for Dma {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.reg = bus.io(IO::DMA);
    }
}
