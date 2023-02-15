#![feature(trait_upcasting)]
#![feature(exclusive_range_pattern)]

mod hram;
pub mod oam;
mod vram;
mod wram;

pub mod mbc;

pub use hram::Hram;
pub use oam::Oam;
pub use vram::Vram;
pub use wram::Wram;
