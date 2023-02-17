#![feature(drain_filter)]

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use shared::{egui::Context, Ui, cpu::*, breakpoints::{Breakpoint, Breakpoints}};

mod disassembly;
mod memory;
mod render;

use disassembly::Disassembly;
use shared::egui::{TextureHandle, TextureId};
use shared::mem::{IOBus};

pub trait Emulator: ReadAccess + Schedule { }
pub trait Bus: shared::cpu::Bus + IOBus { }

pub trait BusWrapper {
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> &mem::mbc::Controller;
}

impl<E: ReadAccess + Schedule> Emulator for E { }
impl<B: shared::cpu::Bus + IOBus> Bus for B { }

pub trait Schedule {
    fn breakpoints(&self) -> Breakpoints;
    fn play(&self);
    fn reset(&self);
}

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
    fn bus(&self) -> Ref<dyn BusWrapper>;
}

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
enum Texture {
    Play,
    Pause,
    Step,
    Reset
}

/// Ninja: Debugger internal code name.
struct Ninja<E: Emulator> {
    emu: E,
    render_data: render::Data,
    disassembly: Disassembly,
    viewer: memory::Viewer,
    textures: HashMap<Texture, TextureHandle>,
    breakpoints: Breakpoints
}

impl<E: Emulator> Ninja<E> {
    pub fn new(emu: E) -> Self {
        Self {
            textures: Default::default(),
            render_data: Default::default(),
            disassembly: Disassembly::new(),
            breakpoints: emu.breakpoints(),
            viewer: memory::Viewer::default(),
            emu,
        }
    }

    pub fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

    pub fn pause(&self) { self.breakpoints.pause(); }
    pub fn step(&self) { self.breakpoints.step(); self.emu.play(); }
    pub fn schedule(&self, bp: Breakpoint) { self.breakpoints.schedule(bp); }
    pub fn breakpoints(&self) -> RefMut<Vec<Breakpoint>> {
        self.breakpoints.bp_mut()
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

