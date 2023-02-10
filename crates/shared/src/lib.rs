pub use egui; //re export egui

pub mod mem;
mod opcodes;
mod registers;
mod value;
pub mod rom;

pub enum Target {
    GB,
    GBC
}

pub mod cpu {
    pub use super::opcodes::*;
    pub use super::registers::{Reg, regs};
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
    fn draw(&mut self, ctx: &egui::Context) { }
}

impl Ui for () { }
