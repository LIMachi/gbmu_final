#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::collections::HashMap;
use std::marker::PhantomData;
use lcd::Lcd;
use shared::mem::{Device, IOBus, Lock, Mem};

mod render;
mod ppu;

mod dma;
mod hdma;

pub use dma::Dma;
pub use hdma::Hdma;
use mem::{Oam, Vram};
use shared::emulator::Emulator;
use shared::io::IORegs;

pub use render::{PpuAccess, VramAccess, VramViewer};
pub use ppu::Ppu;

pub struct Controller {
    ppu: Ppu,
    state: ppu::PpuState,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            state: Ppu::default_state()
        }
    }

    pub fn tick(&mut self, io: &mut IORegs, oam: &mut Lock<Oam>, vram: &mut Lock<Vram>, lcd: &mut Lcd) {
        self.ppu.claim(oam, vram);
        self.ppu.tick(&mut self.state, io, lcd);
        self.ppu.release();
    }

    pub fn inner(&self) -> &Ppu { &self.ppu }
}

impl Device for Controller {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ppu.configure(bus);
    }
}
