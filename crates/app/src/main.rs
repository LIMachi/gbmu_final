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
use shared::utils::clock::{Chrono, Clock};
use crate::app::Emu;
use crate::render::RawContext;

pub struct App<E: 'static> {
    event_loop: Option<EventLoop<E>>,
    pub windows: Windows<E>
}

impl<E> App<E> {
    pub fn new() -> Self {
        let e = EventLoopBuilder::with_user_event()
            .build();
        Self {
            event_loop: Some(e),
            windows: Windows::new(),
        }
    }

    pub fn create<const W: u32, const H: u32, C: 'static + Sized + render::Context<Event = E>, F: 'static + FnOnce(&Instance, Window, &EventLoop<E>) -> C>
        (mut self, handle: Handle, builder: F) -> Self {
        let win = WindowBuilder::new()
            .with_title("GBMU")
            .with_min_inner_size(PhysicalSize::new(160, 144))
            .with_inner_size(PhysicalSize::<u32>::from((W, H)))
            .build(self.event_loop.as_ref().unwrap()).unwrap();
        self.windows.create(self.event_loop.as_ref().unwrap(), win, handle, builder);
        self
    }

    pub fn custom_window<C: 'static + Sized + render::Context<Event = E>, F: 'static + FnOnce(&Instance, Window, &EventLoop<E>) -> C>
        (mut self, builder: WindowBuilder, handle: Handle, handler: F) -> Self {
        let win = builder.build(self.event_loop.as_ref().unwrap()).unwrap();
        self.windows.create(self.event_loop.as_ref().unwrap(), win, handle, handler);
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
    let mut st = Chrono::new();
    let mut current = std::time::Instant::now();
    let mut s = 0;
    let mut acc = 0.0;
    let mut cycles = 0;
    let mut clock = Clock::new(4);
    let debug_window = WindowBuilder::new()
        .with_title("GBMU - debugger")
        .with_min_inner_size(PhysicalSize::<u32>::from((1000, 750)))
        .with_inner_size(PhysicalSize::new(1280, 720));
    app
        .create::<1280, 1152, _, _>(Handle::Main, RawContext::builder(emu.clone()))
        .custom_window(debug_window, Handle::Debug, EguiContext::builder(dbg.clone()))
        .run(move |app| {
            if emu.is_running() {
                if st.paused() {
                    current = std::time::Instant::now();
                    st.start();
                }
                acc += current.elapsed().as_secs_f64();
                current = std::time::Instant::now();
                while acc >= Emu::CYCLE_TIME {
                    cycles += 1;
                    clock.tick();
                    if !emu.cycle(clock.value()) {
                        st.pause();
                        break ;
                    }
                    acc -= Emu::CYCLE_TIME;
                }
            }
            if s != st.elapsed().as_secs() {
                s = st.elapsed().as_secs();
                log::debug!("cycles: {}", cycles as f64 / st.elapsed().as_secs_f64());
            }
        });
}
