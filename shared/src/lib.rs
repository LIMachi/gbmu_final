pub use egui; //re export egui

mod opcodes;
mod registers;
mod value;

pub enum Target {
    GB,
    GBC
}

pub mod cpu {
    pub use super::opcodes::*;
    pub use super::registers::Reg;
    pub use super::value::Value;
}


pub trait Ui {
    fn draw(&mut self, ctx: &egui::Context) { }
}

impl Ui for () { }
