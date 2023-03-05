use shared::io::{IO, IOReg};
use shared::mem::{Device, IOBus};
use shared::utils::FEdge;

#[derive(Default)]
pub struct Timer {
    timer: u8,
    div: IOReg,
    tma: IOReg,
    tima: IOReg,
    tac: IOReg,
    tima_inner: u8,
    tma_inner: u8,
    tima_fedge: FEdge,
    int_flags: IOReg,
    tima_overflow: bool,
}

impl Timer {
    pub fn offset(&mut self) {
        self.div.direct_write(0xAC);
    }

    pub fn tick(&mut self) {
        if self.div.dirty() {
            self.div.reset_dirty();
            self.div.direct_write(0);
        }
        let (v, c) = self.timer.overflowing_add(1);
        self.timer = v;
        let mut d = self.div.value();
        if c {
            d = d.wrapping_add( 1);
            self.div.direct_write(d);
        }
        let tac = self.tac.value();
        let edge = (match tac & 0x3 {
            0 => d >> 1,
            1 => self.timer >> 3,
            2 => self.timer >> 5,
            3 => self.timer >> 7,
            _ => unreachable!()
        } & 0x1) & (tac >> 2);
        let inc = self.tima_fedge.tick(edge != 0);
        let (tima, c) = self.tima_inner.overflowing_add(1);
        let tima = if self.tima.dirty() { self.tma.value() } else { tima };
        let tima = if self.tima_overflow && !self.tima.dirty() { self.tma_inner } else { tima };
        if inc || self.tima.dirty() || (self.tima_overflow && !self.tima.dirty()) {
            self.tima_inner = tima;
        }
        if self.tima_overflow && !self.tima.dirty() {
            self.int_flags.set(2);
        }
        self.tima_overflow = c;
        self.tima.direct_write(self.tima_inner);
        self.tma_inner = self.tma.value();
        self.tima.reset_dirty();
        self.tma.reset_dirty();
    }
}

impl Device for Timer {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.div = bus.io(IO::DIV);
        self.tma = bus.io(IO::TMA);
        self.tima = bus.io(IO::TIMA);
        self.tac = bus.io(IO::TAC);
        self.int_flags = bus.io(IO::IF);
    }
}
