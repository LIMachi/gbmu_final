use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

use shared::io::{IO, IORegs, LCDC};

use crate::ppu::pixel::Attributes;

use super::pixel::Pixel;

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjFifo {
    inner: VecDeque<Pixel>,
    opri: bool,
}

trait Fifo {
    fn push(&mut self, data: Vec<Pixel>) -> bool;
}

impl ObjFifo {
    pub fn new(opri: bool) -> Self {
        ObjFifo { inner: VecDeque::with_capacity(8), opri }
    }

    pub fn pop(&mut self) -> Option<Pixel> {
        self.inner.pop_front()
    }

    pub fn merge(&mut self, data: impl Iterator<Item=Pixel>) -> bool {
        let same_x = self.inner.len() == 8;
        for _ in self.inner.len()..8 {
            self.inner.push_back(Pixel::bg(0, Attributes::default()));
        }
        self.inner
            .iter_mut()
            .zip(data)
            .for_each(|(obj, p)| { obj.mix(p, same_x || !self.opri); });
        true
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BgFifo {
    inner: VecDeque<Pixel>,
    enabled: bool,
}

impl BgFifo {
    pub fn new() -> Self {
        BgFifo {
            enabled: false,
            inner: VecDeque::with_capacity(16),
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn enabled(&self) -> bool { self.enabled }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.disable();
    }

    pub(crate) fn mix(&mut self, oam: &mut ObjFifo, io: &mut IORegs) -> Option<Pixel> {
        if self.enabled {
            let cgb = io.io(IO::CGB).value() != 0;
            let res = match (oam.pop(), self.inner.pop_front()) {
                (None, Some(bg)) => Some(bg),
                (Some(oam), Some(bg)) => Some({
                    if oam.color == 0x0 { bg } else if !cgb && oam.attrs.priority() && bg.color != 0 { bg } else if cgb && io.io(IO::LCDC).value().priority() && bg.color != 0 && (oam.attrs.priority() || bg.attrs.priority()) { bg } else { oam }
                }),
                (_, None) => unreachable!()
            };
            if self.inner.len() <= 8 {
                self.disable();
            }
            res
        } else { None }
    }

    pub fn push(&mut self, data: impl Iterator<Item=Pixel>) -> bool {
        if self.inner.len() > 8 { return false; };
        for pix in data {
            self.inner.push_back(pix);
        }
        if self.inner.len() > 8 { self.enable(); }
        true
    }
}
