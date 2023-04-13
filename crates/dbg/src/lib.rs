#![feature(drain_filter)]
#![feature(if_let_guard)]

use std::cell::{RefMut};
use std::collections::{HashMap, HashSet};
use shared::{egui::Context, Ui, cpu::{Reg, Value}, emulator::Emulator, breakpoints::Breakpoint};

mod render;

use disassembly::Disassembly;
use shared::egui::{TextureHandle, TextureId};
use shared::mem::{IOBus, MBCController};
use shared::winit::event::VirtualKeyCode;

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
enum Texture {
    Play,
    Pause,
    Step,
    Reset,
    Into
}

impl<E: Emulator> Ninja<E> {

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
pub struct Ninja<E: Emulator> {
    render_data: render::Data,
    disassembly: Disassembly<E>,
    viewer: memory::Viewer,
    textures: HashMap<Texture, TextureHandle>,
    keys: HashSet<VirtualKeyCode>,
}
