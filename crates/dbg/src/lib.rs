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

trait ImageLoader {
    fn load_image<S: Into<String>, P: AsRef<std::path::Path>>(&mut self, name: S, path: P) -> TextureHandle;
    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle;
}

impl ImageLoader for Context {
    fn load_image<S: Into<String>, P: AsRef<std::path::Path>>(&mut self, name: S, path: P) -> TextureHandle {
        let img = shared::utils::image::load_image_from_path(path.as_ref()).unwrap();
        self.load_texture(name, img, TextureOptions::LINEAR)
    }

    fn load_svg<const W: u32, const H: u32>(&mut self, name: impl Into<String>, path: impl AsRef<Path>) -> TextureHandle {
        let img = shared::utils::image::load_svg_from_path::<W, H>(path.as_ref()).unwrap();
        self.load_texture(name, img, TextureOptions::LINEAR)
    }
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

