#![feature(slice_flatten)]
#![feature(generic_const_exprs)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lcd::Lcd;
use shared::mem::{*, Device, IOBus, Mem, PPU};
use shared::utils::Cell;

mod render;
mod ppu;

pub struct Controller {
    tab: render::Tabs,
    init: bool,
    clock: usize,
    storage: HashMap<render::Textures, shared::egui::TextureHandle>,
    ppu: Rc<RefCell<ppu::Ppu>>
}

impl Controller {
    pub fn new(lcd: Lcd) -> Self {
        Self {
            tab: render::Tabs::Oam,
            clock: 0,
            init: false,
            storage: HashMap::with_capacity(256),
            ppu: ppu::Ppu::new(lcd).cell()
        }
    }

    pub fn tick(&mut self) {
        self.ppu.as_ref().borrow_mut().tick();
    }
}

impl Device for Controller {
    fn configure(&mut self, bus: &dyn IOBus) {
        self.ppu.as_ref().borrow_mut().configure(bus);
    }
}

impl PPU for Controller {
    fn vram(&self) -> Rc<RefCell<dyn Mem>> {
        self.ppu.clone()
    }

    fn oam(&self) -> Rc<RefCell<dyn Mem>> {
        self.ppu.clone()
    }
}
