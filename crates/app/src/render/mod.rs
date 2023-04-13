use wgpu::Instance;
use winit::window::Window;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowBuilder;
use dbg::Ninja;

mod egui_context;
mod raw_context;

pub mod windows;

pub type Event<'a> = winit::event::Event<'a, Events>;
pub type Proxy = winit::event_loop::EventLoopProxy<Events>;
pub type EventLoop = winit::event_loop::EventLoop<Events>;

pub trait Context<Ctx> {
    fn inner(&mut self) -> &mut Window;
    fn redraw(&mut self, ctx: &mut Ctx);
    fn request_redraw(&mut self);

    fn resize(&mut self, physical: PhysicalSize<u32>, emu: &mut Ctx);
    fn data(&mut self) -> Box<&mut dyn std::any::Any>;

    fn handle(&mut self, event: &Event, ctx: &mut Ctx);
}

pub trait Render {

    fn init(&mut self, window: &Window, emu: &mut Emulator);
    fn render(&mut self, emu: &mut Emulator);
    fn resize(&mut self, w: u32, h: u32, emu: &mut Emulator);
    fn handle(&mut self, event: &Event, window: &Window, emu: &mut Emulator);
}

pub use egui_context::EguiContext;
pub use raw_context::RawContext;
use shared::{Events, Handle};
use crate::app::Menu;
use crate::emulator::{Emulator, Screen};
use crate::settings::Settings;

pub struct WindowType(Handle);

impl WindowType {
    pub fn build(&self, evt: &EventLoopWindowTarget<Events>) -> Window {
        match self.0 {
            Handle::Main => WindowBuilder::new()
                .with_title("GBMU")
                .with_min_inner_size(PhysicalSize::new(800, 600)),
            Handle::Sprites => WindowBuilder::new()
                .with_title("GBMU - Spritesheet")
                .with_min_inner_size(PhysicalSize::new(1200, 860))
                .with_resizable(false),
            Handle::Game => WindowBuilder::new()
                .with_title("GBMU")
                .with_min_inner_size(PhysicalSize::new(160, 144))
                .with_inner_size(PhysicalSize::<u32>::from((640, 576))),
            Handle::Debug => WindowBuilder::new()
                .with_title("GBMU - debugger")
                .with_min_inner_size(PhysicalSize::<u32>::from((1000, 750)))
                .with_inner_size(PhysicalSize::new(1280, 720)),
            Handle::Settings => WindowBuilder::new()
                .with_title("GBMU - settings")
                .with_inner_size(PhysicalSize::<u32>::from((240, 800)))
                .with_resizable(false),
        }.build(evt).unwrap()
    }

    pub fn builder(self, emu: &mut Emulator) -> Box<dyn FnOnce(&Instance, Window, Proxy) -> Box<dyn Context<Emulator>> + '_> {
        match self.0 {
            Handle::Main => EguiContext::<Emulator, Menu>::builder(emu),
            Handle::Game => RawContext::<Screen>::builder(emu),
            Handle::Debug => EguiContext::<Emulator, Ninja<Emulator>>::builder(emu),
            Handle::Sprites => EguiContext::<Emulator, ppu::VramViewer<Emulator>>::builder(emu),
            Handle::Settings => EguiContext::<Emulator, Settings>::builder(emu),
        }
    }
}
