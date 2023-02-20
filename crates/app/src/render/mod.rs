use std::ops::Deref;
use wgpu::Instance;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowBuilder;
use dbg::Debugger;

mod egui_context;
mod raw_context;

pub mod windows;

#[derive(Clone)]
pub enum WindowType {
    Main(Menu), // debugger / library
    Debug(Debugger<Emulator>),
    Game(Emulator),
    Keybindings
}

impl WindowType {
    pub fn handle(&self) -> Handle {
        match self {
            WindowType::Main(_) => Handle::Main,
            WindowType::Game(_) => Handle::Game,
            WindowType::Debug(_) => Handle::Debug,
            WindowType::Keybindings => Handle::Settings
        }
    }

    pub fn build(&self, evt: &EventLoopWindowTarget<Events>) -> Window {
        match self {
            WindowType::Main(_) => WindowBuilder::new()
                .with_title("GBMU")
                .with_min_inner_size(PhysicalSize::new(800, 600)),
            WindowType::Game(_) => WindowBuilder::new()
                .with_title(" ")
                .with_min_inner_size(PhysicalSize::new(160, 144))
                .with_inner_size(PhysicalSize::<u32>::from((640, 576))),
            WindowType::Debug(_) => WindowBuilder::new()
                .with_title("GBMU - debugger")
                .with_min_inner_size(PhysicalSize::<u32>::from((1000, 750)))
                .with_inner_size(PhysicalSize::new(1280, 720)),
            WindowType::Keybindings => unimplemented!()
        }.build(evt).unwrap()
    }

    pub fn ctx(self) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context>> {
        match self {
            WindowType::Main(menu) => EguiContext::builder(menu),
            WindowType::Game(emu) => RawContext::builder(emu),
            WindowType::Debug(ninja) => EguiContext::builder(ninja),
            _ => unimplemented!()
        }
    }
}

pub type Event<'a> = winit::event::Event<'a, Events>;
pub type Proxy = winit::event_loop::EventLoopProxy<Events>;
pub type EventLoop = winit::event_loop::EventLoop<Events>;

pub type Flow = std::ops::ControlFlow<()>;
pub const CONTINUE: Flow = Flow::Continue(());
pub const BREAK: Flow = Flow::Break(());

pub trait Context {
    fn inner(&mut self) -> &mut Window;
    fn redraw(&mut self) -> Flow;
    fn request_redraw(&mut self);

    fn resize(&mut self, physical: PhysicalSize<u32>);
    fn data(&mut self) -> Box<&mut dyn std::any::Any>;

    fn handle(&mut self, event: &Event);
}

pub trait Render {

    fn init(&mut self, window: &Window);
    fn render(&mut self);
    fn resize(&mut self, w: u32, h: u32);
    fn handle(&mut self, event: &Event, proxy: &Proxy, window: &Window);
}

pub use egui_context::EguiContext;
pub use raw_context::RawContext;
use shared::cpu::Reg;
use shared::{Events, Handle};
use shared::rom::Rom;
use crate::emulator::Emulator;
use crate::Menu;
