#![feature(if_let_guard)]
#![feature(drain_filter)]
#![feature(hash_drain_filter)]


pub use egui;
pub use winit;
use crate::cpu::Opcode;

pub mod events {
    pub use super::winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
}

pub mod utils;
pub mod mem;
pub mod io;

pub mod rom;
pub mod breakpoints;
pub use serde;

pub mod input;
pub mod audio_settings;

mod opcodes;
mod registers;
mod value;

#[derive(Copy, Debug, Eq, PartialEq, Hash, Clone)]
pub enum Handle {
    Main,
    Debug,
    Game,
    Sprites,
    Settings
}

pub type Event<'a> = winit::event::Event<'a, Events>;

#[derive(Debug)]
pub enum Events {
    Play(rom::Rom),
    Reload,
    Load(String),
    Loaded,
    Open(Handle),
    Close
}

pub enum Target {
    GB,
    GBC
}

pub trait Cpu {
    fn done(&self) -> bool;

    fn previous(&self) -> Opcode;
    fn register(&self, reg: registers::Reg) -> value::Value;
}

pub mod cpu {
    pub use super::opcodes::*;
    pub use super::registers::{Reg, regs, Flags};
    pub use super::value::Value;

    pub trait Bus {
        fn status(&self) -> MemStatus;
        fn update(&mut self, status: MemStatus);
        fn get_range(&self, start: u16, len: u16) -> Vec<u8>;
        fn write(&mut self, addr: u16, value: u8);

        /// Bypasses read cycle
        /// CPU doesn't use this
        fn direct_read(&self, offset: u16) -> u8;
        fn int_reset(&mut self, bit: u8);
        fn int_set(&mut self, bit: u8);
        fn interrupt(&self) -> u8;
    }

    #[derive(Copy, Debug, Clone, Eq, PartialEq)]
    pub enum MemStatus {
        Read(u8),
        Write(u16),
        Ready,
        ReqRead(u16),
        ReqWrite(u16),
        Idle
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Op {
        Read(u16, u8),
        Write(u16, u8)
    }

}

pub trait Ui {
    type Context;

    fn init(&mut self, ctx: &mut egui::Context, ext: &mut Ui::Context) { }
    fn draw(&mut self, ctx: &mut egui::Context, ext: &mut Ui::Context) { }
    fn handle(&mut self, event: &winit::event::Event<Events>, ext: &mut Ui::Context) { }
}

impl Ui for () {
    type Context = ();
}
