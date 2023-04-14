#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::collections::HashMap;
use std::marker::PhantomData;
use lcd::Lcd;
use shared::mem::Lock;

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

    pub fn tick<'a>(&mut self, io: &mut IORegs, oam: &'a mut Lock<Oam>, vram: &'a mut Lock<Vram>, lcd: &mut Lcd) {
        self.ppu.claim(oam, vram);
        self.ppu.tick(&mut self.state, io, lcd);
        self.ppu.release();
    }

    pub fn inner(&self) -> &Ppu { &self.ppu }
}
