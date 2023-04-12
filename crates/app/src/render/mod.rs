use wgpu::Instance;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowBuilder;
use dbg::Debugger;

mod egui_context;
mod raw_context;

pub mod windows;

// TODO make those not depend on Emulator, it will be the ext context
// the internal type is already there for some (Settings, Debug), need a new struct for the others
#[derive(Clone)]
pub enum WindowType<'a> {
    Main(&'a mut Menu), // debugger / library
    Debug(&'a mut Debugger<Emulator>),
    Game(&'a mut Emulator),
    Sprites(&'a mut Emulator),
    Settings(&'a mut Emulator)
}

impl<'a> WindowType<'a> {
    pub fn handle(&self) -> Handle {
        match self {
            WindowType::Main(_) => Handle::Main,
            WindowType::Game(_) => Handle::Game,
            WindowType::Debug(_) => Handle::Debug,
            WindowType::Sprites(_) => Handle::Sprites,
            WindowType::Settings(_) => Handle::Settings
        }
    }

    pub fn build(&self, evt: &EventLoopWindowTarget<Events>) -> Window {
        match self {
            WindowType::Main(_) => WindowBuilder::new()
                .with_title("GBMU")
                .with_min_inner_size(PhysicalSize::new(800, 600)),
            WindowType::Sprites(_) => WindowBuilder::new()
                .with_title("GBMU - Spritesheet")
                .with_min_inner_size(PhysicalSize::new(1200, 860))
                .with_resizable(false),
            WindowType::Game(_) => WindowBuilder::new()
                .with_title(" ")
                .with_min_inner_size(PhysicalSize::new(160, 144))
                .with_inner_size(PhysicalSize::<u32>::from((640, 576))),
            WindowType::Debug(_) => WindowBuilder::new()
                .with_title("GBMU - debugger")
                .with_min_inner_size(PhysicalSize::<u32>::from((1000, 750)))
                .with_inner_size(PhysicalSize::new(1280, 720)),
            WindowType::Settings(_) => WindowBuilder::new()
                .with_title("GBMU - settings")
                .with_inner_size(PhysicalSize::<u32>::from((240, 800)))
                .with_resizable(false),
        }.build(evt).unwrap()
    }

    // TODO add types to builders, menu/emu/ninja are already good (they're the external context in this case)
    pub fn ctx(self) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context<Emulator>>> {
        match self {
            WindowType::Main(menu) => EguiContext::builder(menu),
            WindowType::Game(emu) => RawContext::builder(emu),
            WindowType::Debug(ninja) => EguiContext::builder(ninja),
            WindowType::Sprites(emu) => EguiContext::builder(emu),
            WindowType::Settings(emu) => EguiContext::builder(emu.settings()),
        }
    }
}

pub type Event<'a> = winit::event::Event<'a, Events>;
pub type Proxy = winit::event_loop::EventLoopProxy<Events>;
pub type EventLoop = winit::event_loop::EventLoop<Events>;

pub trait Context<Ctx> {
    fn inner(&mut self) -> &mut Window;
    fn redraw(&mut self, ctx: &mut Ctx);
    fn request_redraw(&mut self);

    fn resize(&mut self, physical: PhysicalSize<u32>);
    fn data(&mut self) -> Box<&mut dyn std::any::Any>;

    fn handle(&mut self, event: &Event, ctx: &mut Ctx);
}

pub trait Render {

    fn init(&mut self, window: &Window);
    fn render(&mut self);
    fn resize(&mut self, w: u32, h: u32);
    fn handle(&mut self, event: &Event, window: &Window);
}

pub use egui_context::EguiContext;
pub use raw_context::RawContext;
use shared::{Events, Handle};
use crate::emulator::Emulator;
use crate::Menu;
