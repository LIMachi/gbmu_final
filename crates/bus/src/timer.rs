use std::time::Instant;
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
    tima_fedge: FEdge,
    int_flags: IOReg,
    tima_overflow: bool,
}

impl Timer {
    pub fn offset(&mut self) {
        self.div.direct_write(0xAC);
    }

    // TODO doesnt tick during cpu stop mode
    pub fn tick(&mut self) {
        let (v, c) = if self.div.dirty() {
            self.div.reset_dirty();
            self.div.direct_write(0);
            (0, false)
        } else { self.timer.overflowing_add(4) };
        self.timer = v;
        let mut d = self.div.value();
        if c {
            let (d, o) = d.overflowing_add( 1);
            self.div.direct_write(d);
        }
        let tac = self.tac.value();
        let tac_enable = tac & 4 != 0;
        let edge = tac_enable && (match tac & 0x3 {
            0 => self.div.value() >> 1,
            1 => self.timer >> 3,
            2 => self.timer >> 5,
            3 => self.timer >> 7,
            _ => unreachable!()
        } & 0x1) !=0;
        let (mut tima, mut c) = self.tima_inner.overflowing_add(self.tima_fedge.tick(edge) as u8);
        if self.tima.dirty() {
            c = false;
            tima = self.tima.value();
            self.tima.reset_dirty();
        }
        if self.tima_overflow { tima = self.tma.value() };
        self.tima.direct_write(tima);
        if self.tima_overflow { self.int_flags.set(2); }
        self.tima_overflow = c;
        self.tima_inner = tima;
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
