#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::collections::HashMap;
use std::marker::PhantomData;

pub use dma::Dma;
pub use hdma::Hdma;
use lcd::Lcd;
use mem::{Oam, Vram};
pub use ppu::Ppu;
pub use render::{PpuAccess, VramAccess, VramViewer};
use shared::emulator::Emulator;
use shared::io::{IO, IODevice, IORegs};
use shared::mem::{IOBus, Lock};

mod render;
mod ppu;

mod dma;
mod hdma;

pub struct Controller {
    ppu: Ppu,
    state: ppu::PpuState,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            state: Ppu::default_state(),
        }
    }

    pub fn tick<'a>(&mut self, io: &mut IORegs, oam: &'a mut Lock<Oam>, vram: &'a mut Lock<Vram>, lcd: &mut Lcd) {
        self.ppu.claim(oam, vram);
        self.ppu.tick(&mut self.state, io, lcd);
        self.ppu.release();
    }

    pub fn inner(&self) -> &Ppu { &self.ppu }
}

impl IODevice for Controller {
    fn write(&mut self, io: IO, v: u8, bus: &mut dyn IOBus) {
        IODevice::write(&mut self.ppu, io, v, bus);
    }
}
