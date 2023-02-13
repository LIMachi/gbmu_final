use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use shared::{egui::Context, Ui, cpu::*, Break};

mod disassembly;
mod render;

use disassembly::Disassembly;
use shared::egui::{TextureHandle, TextureId, TextureOptions};

pub trait Emulator: ReadAccess + Schedule { }

impl<E: ReadAccess + Schedule> Emulator for E { }

pub trait Schedule {
    fn schedule_break(&mut self, bp: Break) -> &mut Self;
    fn pause(&mut self);
    fn play(&mut self);
    fn reset(&mut self);
}

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
}

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
enum Texture {
    Play,
    Pause,
    Step
}

/// Ninja: Debugger internal code name.
struct Ninja<E: Emulator> {
    emu: E,
    disassembly: Disassembly,
    textures: HashMap<Texture, TextureHandle>
}

impl<E: Emulator> Ninja<E> {
    pub fn new(emu: E) -> Self {
        Self {
            textures: Default::default(),
            emu,
            disassembly: Disassembly::new()
        }
    }

    pub fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

}

#[derive(Clone)]
pub struct Debugger<E: Emulator> {
    inner: Rc<RefCell<Ninja<E>>>
}

impl<E:Emulator> Ui for Debugger<E> {
    fn init(&mut self, ctx: &mut Context) {
        self.inner.borrow_mut().init(ctx);
    }

    fn draw(&mut self, ctx: &Context) {
        self.inner.borrow_mut().draw(ctx)
    }
}

impl<E: Emulator> Debugger<E> {
    pub fn new(emu: E) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Ninja::new(emu)))
        }
    }

}

