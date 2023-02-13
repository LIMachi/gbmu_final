use std::borrow::BorrowMut;
pub use egui;
pub use winit;

pub mod utils;
pub mod mem;
pub mod io;
mod opcodes;
mod registers;
mod value;
pub mod rom;

pub enum Target {
    GB,
    GBC
}

pub trait Cpu {
    fn register(&self, reg: registers::Reg) -> value::Value;
}

pub enum Break {
    Instructions(usize),
    Register(registers::Reg, value::Value)
}

impl Break {
    pub fn tick(&mut self, runner: &impl Cpu) -> bool {
        match self {
            Break::Instructions(n) if *n <= 1 => true,
            Break::Instructions(n) => { *n = *n - 1; false },
            Break::Register(r, v) if runner.register(*r) == *v => true,
            _ => false
        }
    }

    pub fn address(addr: u16) -> Self {
        Self::Register(registers::Reg::PC, addr.into())
    }
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
    fn draw(&mut self, ctx: &egui::Context) { }
}

impl Ui for () { }
