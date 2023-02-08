pub mod inc;
pub mod dec;
pub mod read;
pub mod write;
pub mod mem;

pub mod alu;
pub mod bits;

pub mod cond;
pub mod int;

use super::*;

pub type Flow = std::ops::ControlFlow<(), ()>;
pub type Op = fn(&mut State) -> Flow;

pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());
