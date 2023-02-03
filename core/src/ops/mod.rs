pub mod math;
pub mod read;
pub mod write;

use super::*;

pub type Flow = std::ops::ControlFlow<(), ()>;
pub type Op = fn(&mut Cpu, &mut State) -> Flow;

pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());
