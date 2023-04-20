#![feature(if_let_guard)]
#![feature(drain_filter)]
#![feature(hash_drain_filter)]


pub use egui;
pub use serde;
pub use winit;

use crate::input::KeyCat;

pub mod events {
    pub use super::winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
}

pub mod widgets;
pub mod utils;
pub mod mem;
pub mod io;

pub mod rom;
pub mod breakpoints;

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
    Settings,
}

pub type Event<'a> = winit::event::Event<'a, Events>;

#[derive(Debug)]
pub enum Events {
    Play(rom::Rom),
    Reload,
    Load(String),
    Loaded,
    Open(Handle),
    AudioSwitch,
    Press(KeyCat),
    Release(KeyCat),
    Close,
}

pub enum Target {
    GB,
    GBC,
}

pub trait Ui {
    type Ext;

    fn init(&mut self, _ctx: &mut egui::Context, _ext: &mut <Self as Ui>::Ext) {}
    fn draw(&mut self, _ctx: &mut egui::Context, _ext: &mut <Self as Ui>::Ext) {}
    fn handle(&mut self, _event: &winit::event::Event<Events>, _ext: &mut <Self as Ui>::Ext) {}
}

impl Ui for () {
    type Ext = ();
}
