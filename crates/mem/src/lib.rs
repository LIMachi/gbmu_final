#![feature(trait_upcasting)]
#![feature(exclusive_range_pattern)]
#![feature(drain_filter)]

pub mod oam;
mod hram;
mod vram;
mod wram;

mod boot;

pub mod mbc;

pub use hram::Hram;
pub use oam::Oam;
pub use vram::Vram;
pub use wram::Wram;
