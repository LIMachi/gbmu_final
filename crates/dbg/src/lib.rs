#![feature(drain_filter)]
#![feature(if_let_guard)]

use std::cell::{Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use shared::{egui::Context, Ui, cpu::{self, Reg, Value}, breakpoints::{Breakpoint, Breakpoints}, Event};

mod disassembly;
mod memory;
mod render;

use disassembly::Disassembly;
use shared::egui::{TextureHandle, TextureId};
use shared::input::Section;
use shared::mem::{IOBus, MBCController};
use shared::winit::event::VirtualKeyCode;

pub trait Emulator: ReadAccess + Schedule { }
pub trait Bus: cpu::Bus + IOBus { }

pub trait BusWrapper {
    fn bus(&self) -> Box<&dyn Bus>;
    fn mbc(&self) -> Ref<dyn MBCController>;
}

impl<E: ReadAccess + Schedule> Emulator for E { }
impl<B: cpu::Bus + IOBus> Bus for B { }

pub trait Schedule {
    fn breakpoints(&self) -> Breakpoints;
    fn play(&self);
    fn reset(&self);

    fn speed(&self) -> i32;
    fn set_speed(&self, speed: i32);
}

pub trait ReadAccess {
    fn cpu_register(&self, reg: Reg) -> Value;
    fn get_range(&self, st: u16, len: u16) -> Vec<u8>;
    fn bus(&self) -> Ref<dyn BusWrapper>;
    fn binding(&self, key: VirtualKeyCode) -> Option<Section>;
}

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
enum Texture {
    Play,
    Pause,
    Step,
    Reset,
    Into
}

/// Ninja: Debugger internal code name.
struct Ninja<E: Emulator> {
    emu: E,
    render_data: render::Data,
    disassembly: Disassembly<E>,
    viewer: memory::Viewer,
    textures: HashMap<Texture, TextureHandle>,
    keys: HashSet<VirtualKeyCode>,
    breakpoints: Breakpoints
}

impl<E: Emulator> Ninja<E> {
    pub fn new(emu: E) -> Self {
        Self {
            textures: Default::default(),
            render_data: Default::default(),
            disassembly: Disassembly::new(),
            breakpoints: emu.breakpoints(),
            keys: Default::default(),
            viewer: memory::Viewer::new(emu.breakpoints()),
            emu,
        }
    }

    pub fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

    pub fn pause(&self) { self.breakpoints.pause(); }

    pub fn play(&mut self) {
        self.emu.play();
        self.disassembly.follow();
    }

    pub fn reset(&self) {
        self.emu.reset();
    }

    pub fn step(&mut self) {
        if let Some((pc, op)) = self.disassembly.next(&self.emu) {
            if op.is_jmp() { self.breakpoints.step() }
            else { self.breakpoints.schedule(Breakpoint::register(Reg::PC, Value::U16(pc + op.size as u16)).once()) }
        } else { self.breakpoints.step(); }
        self.play();
    }

    pub fn step_into(&mut self) { self.breakpoints.step(); self.play(); }
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

    fn draw(&mut self, ctx: &mut Context) {
        self.inner.borrow_mut().draw(ctx)
    }

    fn handle(&mut self, event: &Event) {self.inner.borrow_mut().handle(event); }
}

impl<E: Emulator> Debugger<E> {
    pub fn new(emu: E) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Ninja::new(emu)))
        }
    }

}

