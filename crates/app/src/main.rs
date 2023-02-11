#![feature(drain_filter)]

use std::borrow::Borrow;
use wgpu::Instance;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::Window
};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopBuilder;
use winit::window::WindowBuilder;
use dbg::Emulator;

mod render;
mod app;

use render::{EguiContext, Handle, windows::Windows};
use shared::Events;

pub struct App<E: 'static> {
    event_loop: Option<EventLoop<E>>,
    pub windows: Windows<E>
}

impl<E> App<E> {
    pub fn new() -> Self {
        let e = EventLoopBuilder::with_user_event().build();
        Self {
            event_loop: Some(e),
            windows: Windows::new(),
        }
    }

    pub fn create<const W: u32, const H: u32, C: 'static + Sized + render::Context<Event = E>, F: 'static + FnOnce(&Instance, Window, &EventLoop<E>) -> C>
        (mut self, handle: Handle, builder: F) -> Self {
        let win = WindowBuilder::new()
            .with_title("GBMU")
            .with_inner_size(PhysicalSize::<u32>::from((W, H)))
            .with_min_inner_size(PhysicalSize::<u32>::from((1000, 750)))
            .build(self.event_loop.as_ref().unwrap()).unwrap();
        self.windows.create(self.event_loop.as_ref().unwrap(), win, handle, builder);
        self
    }

    pub fn run<F: 'static + FnMut(&mut App<E>)>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        event.run(move |mut event: Event<'_, E>, target: &EventLoopWindowTarget<E>, flow: &mut ControlFlow| {
            flow.set_poll();
            self.windows.handle_events(&event, flow);
            match event {
                Event::MainEventsCleared => {
                    handler(&mut self);
                    self.windows.update();
                },
                Event::RedrawEventsCleared => {
                    //TODO wait, so GPU does not burn
                },
                _ => {}
            }
        })
    }
}

fn main() {
    env_logger::init();

    let app = App::<Events>::new();
    let mut emu = app::Emulator::new();
    let dbg = dbg::Debugger::new(emu.clone());
    app.create::<1280, 720, _, _>(Handle::Main, EguiContext::builder(dbg.clone()))
        .run(move |app| {
            emu.cycle();
        });
}
