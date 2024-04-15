use serde::{Deserialize, Serialize};
use shared::io::{IO, IORegs};
use shared::utils::FEdge;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Timer {
    timer: u8,
    tima_inner: u8,
    tima_fedge: FEdge,
    tima_overflow: bool,
}

impl Timer {
    // pub fn offset(&mut self) {
    //     self.div.direct_write(0xAC);
    // }

    // TODO doesnt tick during cpu stop mode
    pub fn tick(&mut self, io: &mut IORegs) {
        let div = io.io_mut(IO::DIV);
        let (v, c) = if div.dirty() {
            div.reset_dirty();
            div.direct_write(0);
            (0, false)
        } else { self.timer.overflowing_add(4) };
        self.timer = v;
        let mut d = div.value();
        if c {
            d = d.wrapping_add(1);
            div.direct_write(d);
        }
        let tac = io.io(IO::TAC).value();
        let tac_enable = tac & 4 != 0;
        let edge = tac_enable && (match tac & 0x3 {
            0 => d >> 1,
            1 => self.timer >> 3,
            2 => self.timer >> 5,
            3 => self.timer >> 7,
            _ => unreachable!()
        } & 0x1) !=0;
        let (mut tima, mut c) = self.tima_inner.overflowing_add(self.tima_fedge.tick(edge) as u8);
        let tma = io.io(IO::TMA).value();
        let io_tima = io.io_mut(IO::TIMA);
        if io_tima.dirty() {
            c = false;
            tima = io_tima.value();
            io_tima.reset_dirty();
        }
        if self.tima_overflow { tima = tma };
        io_tima.direct_write(tima);
        if self.tima_overflow { io.int_set(2); }
        self.tima_overflow = c;
        self.tima_inner = tima;
    }
}
