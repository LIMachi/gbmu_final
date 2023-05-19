#![feature(drain_filter)]
#![feature(if_let_guard)]

use std::collections::HashMap;

use render::{Disassembly, Viewer};
use shared::{breakpoints::Breakpoint, cpu::{Reg, Value}, egui::Context, emulator::Emulator};
use shared::egui::{TextureHandle, TextureId};

mod render;

#[derive(Copy, Clone, Hash, PartialOrd, PartialEq, Eq)]
enum Texture {
    Play,
    Pause,
    Step,
    Reset,
    Into,
}

trait Debugger<E: Emulator> {
    fn pause(&mut self);
    fn play(&mut self, dice: &mut Disassembly<E>);
    fn step(&mut self, dice: &mut Disassembly<E>);
    fn run_to(&mut self, dice: &mut Disassembly<E>, addr: u16);
    fn step_into(&mut self, dice: &mut Disassembly<E>);

    fn schedule(&mut self, bp: Breakpoint);
}

impl<E: Emulator> Debugger<E> for E {
    fn pause(&mut self) {
        self.breakpoints().pause();
    }

    fn play(&mut self, dice: &mut Disassembly<E>) {
        self.play();
        dice.follow();
    }

    fn step(&mut self, dice: &mut Disassembly<E>) {
        if let Some((pc, op)) = dice.next(self) {
            if op.is_jmp() { self.breakpoints().step() } else { self.breakpoints().schedule(Breakpoint::register(Reg::PC, Value::U16(pc + op.size as u16)).once()) }
        } else { self.breakpoints().step(); }
        Debugger::<E>::play(self, dice);
    }

    fn run_to(&mut self, dice: &mut Disassembly<E>, addr: u16) {
        self.breakpoints().schedule(Breakpoint::address(addr).once());
        Debugger::<E>::play(self, dice);
    }

    fn step_into(&mut self, dice: &mut Disassembly<E>) {
        self.breakpoints().step();
        Debugger::<E>::play(self, dice);
    }

    fn schedule(&mut self, bp: Breakpoint) {
        self.breakpoints().schedule(bp);
    }
}

impl<E: Emulator> Ninja<E> {
    pub(crate) fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }
}

pub struct Ninja<E: Emulator> {
    render_data: render::Data,
    disassembly: Disassembly<E>,
    viewer: Viewer,
    textures: HashMap<Texture, TextureHandle>,
}
