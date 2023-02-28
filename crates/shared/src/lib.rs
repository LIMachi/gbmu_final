#![feature(drain_filter)]

use std::borrow::BorrowMut;
pub use egui;
pub use winit;
use crate::cpu::Opcode;

pub mod utils;
pub mod mem;
pub mod io;
mod opcodes;
mod registers;
mod value;
pub mod rom;
pub mod breakpoints;

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
    Close(Handle)
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
        fn tick(&mut self);
        fn get_range(&self, start: u16, len: u16) -> Vec<u8>;
        fn write(&mut self, addr: u16, value: u8);
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
}

pub trait Ui {
    fn init(&mut self, ctx: &mut egui::Context) { }
    fn draw(&mut self, ctx: &mut egui::Context) { }
    fn handle(&mut self, event: &winit::event::Event<Events>) { }
}

impl Ui for () { }
