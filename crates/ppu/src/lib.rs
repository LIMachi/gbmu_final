#![feature(slice_flatten)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(if_let_guard)]

extern crate core;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lcd::Lcd;
use shared::mem::{Device, IOBus, Mem, PPU};

mod render;
mod ppu;

mod dma;
mod hdma;

pub use dma::Dma;
pub use hdma::Hdma;
use crate::render::Textures;

struct UiData {
    textures: HashMap<render::Textures, shared::egui::TextureHandle>,
    bg_data: Option<render::TileData>
}

impl UiData {
    pub fn new() -> Self {
        Self {
            textures: HashMap::with_capacity(256),
            bg_data: None,
        }
    }

    pub fn get(&self, tile: usize) -> shared::egui::TextureHandle {
        self.textures.get(&render::Textures::Tile(tile))
            .or(self.textures.get(&render::Textures::Blank))
            .cloned().unwrap()
    }

    pub fn insert(&mut self, tile: Textures, tex: shared::egui::TextureHandle) {
        self.textures.insert(tile, tex);
    }

    pub fn tex(&self, tex: Textures) -> Option<shared::egui::TextureHandle> {
        self.textures.get(&tex).cloned()
    }
}

pub struct Controller {
    tab: render::Tabs,
    init: bool,
    ui: UiData,
    ppu: ppu::Ppu,
    state: ppu::PpuState,
}

impl Controller {
    pub fn new(lcd: Lcd) -> Self {
        Self {
            tab: render::Tabs::Oam,
            init: false,
            ui: UiData::new(),
            ppu: ppu::Ppu::new(lcd),
            state: ppu::Ppu::default_state()
        }
    }

    pub fn tick(&mut self) {
        self.ppu.tick(&mut self.state);
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
