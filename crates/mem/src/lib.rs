#![feature(trait_upcasting)]
#![feature(exclusive_range_pattern)]
#![feature(drain_filter)]

mod hram;
pub mod oam;
mod vram;
mod wram;
pub mod lock;

pub mod mbc;

pub use hram::Hram;
pub use oam::Oam;
pub use vram::Vram;
pub use wram::Wram;
pub use lock::Lock;
