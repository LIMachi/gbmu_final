#![feature(if_let_guard)]
#![feature(drain_filter)]
#![feature(hash_drain_filter)]


pub use egui;
pub use winit;

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

pub mod cpu;
pub mod emulator;

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

pub trait Ui {
    type Ext;

    fn new(ctx: &mut <Self as Ui>::Ext) -> Self where Self: Sized;

    fn init(&mut self, _ctx: &mut egui::Context, _ext: &mut <Self as Ui>::Ext) { }
    fn draw(&mut self, _ctx: &mut egui::Context, _ext: &mut <Self as Ui>::Ext) { }
    fn handle(&mut self, _event: &winit::event::Event<Events>, _ext: &mut <Self as Ui>::Ext) { }
}

impl Ui for () {
    type Ext = ();

    fn new(_: &mut Self::Ext) -> Self where Self: Sized {
        ()
    }
}
