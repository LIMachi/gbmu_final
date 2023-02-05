pub mod math;
pub mod inc;
pub mod dec;
pub mod read;
pub mod write;
pub mod mem;

use super::*;

pub type Flow = std::ops::ControlFlow<(), ()>;
pub type Op = fn(&mut State) -> Flow;

pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());

pub const INC_PC: &[Op] = &[read::pc, inc::inc16, write::pc];
