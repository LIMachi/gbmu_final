#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lcd::Lcd;
use shared::mem::{Device, IOBus, Lock, Mem, PPU};

mod render;
mod ppu;

mod dma;
mod hdma;

pub use dma::Dma;
pub use hdma::Hdma;
use mem::{Oam, Vram};
use shared::io::IORegs;

pub struct Controller {
    tab: render::Tabs,
    init: bool,
    storage: HashMap<render::Textures, shared::egui::TextureHandle>,
    ppu: ppu::Ppu,
    state: ppu::PpuState,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            tab: render::Tabs::Oam,
            init: false,
            storage: HashMap::with_capacity(256),
            ppu: ppu::Ppu::new(),
            state: ppu::Ppu::default_state()
        }
    }

    pub fn tick(&mut self, io: &mut IORegs, oam: &mut Lock<Oam>, vram: &mut Lock<Vram>, lcd: &mut Lcd) {
        self.ppu.claim(oam, vram);
        self.ppu.tick(&mut self.state, io, lcd);
        self.ppu.release();
    }
}

impl Device for Controller {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ppu.configure(bus);
    }
}

impl PPU for Controller {
    fn vram(&self) -> Rc<RefCell<dyn Mem>> { self.ppu.vram.clone() }
    fn oam(&self) -> Rc<RefCell<dyn Mem>> { self.ppu.oam.clone() }
}
