use wgpu::Instance;
use winit::{window::Window, event_loop::EventLoop};
use winit::dpi::PhysicalSize;

pub mod dbg;
mod egui_context;
mod pixels;

pub mod windows;

use egui_context::Ui;

pub type Flow = std::ops::ControlFlow<()>;
pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());

pub trait Context {
    fn inner(&mut self) -> &mut Window;
    fn redraw(&mut self) -> Flow;
    fn resize(&mut self, physical: PhysicalSize<u32>);
    fn data(&mut self) -> Box<&mut dyn std::any::Any>;
}

pub use egui_context::EguiContext;
pub use windows::Handle;
