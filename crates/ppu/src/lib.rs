#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

extern crate core;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lcd::Lcd;
use shared::mem::{Device, IOBus, Mem, PPU};
use shared::utils::Cell;

mod render;
mod ppu;

mod dma;
mod hdma;

pub use dma::Dma;
pub use hdma::Hdma;

pub struct Controller {
    tab: render::Tabs,
    init: bool,
    storage: HashMap<render::Textures, shared::egui::TextureHandle>,
    ppu: ppu::Ppu
}

impl Controller {
    pub fn new(lcd: Lcd) -> Self {
        Self {
            tab: render::Tabs::Oam,
            init: false,
            storage: HashMap::with_capacity(256),
            ppu: ppu::Ppu::new(lcd)
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick();
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
