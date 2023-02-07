use wgpu::Instance;
use winit::{
    window::{Window},
    event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget},
    event::{Event}
};
use winit::dpi::PhysicalSize;
use winit::window::WindowBuilder;

mod render;
mod app;
use render::{Handle, windows::Windows, dbg::Debugger};
use crate::render::EguiContext;

pub struct App {
    event_loop: Option<EventLoop<()>>,
    pub windows: Windows
}

impl App {
    pub fn new() -> Self {
        Self {
            event_loop: Some(EventLoop::new()),
            windows: Windows::new(),
        }
    }

    pub fn create<const W: u32, const H: u32, C: 'static + Sized + render::Context, F: 'static + FnOnce(&Instance, Window, &EventLoop<()>) -> C>
        (mut self, handle: render::Handle, builder: F) -> Self {
        let win = WindowBuilder::new()
            .with_title("GBMU")
            .with_inner_size(PhysicalSize::<u32>::from((W, H)))
            .build(self.event_loop.as_ref().unwrap()).unwrap();
        self.windows.create(self.event_loop.as_ref().unwrap(), win, handle, builder);
        self
    }

    pub fn run<F: 'static + FnMut(&mut App)>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        let mut start = true;
        event.run(move |mut event: Event<'_, ()>, target: &EventLoopWindowTarget<()>, flow: &mut ControlFlow| {
            flow.set_poll();
            self.windows.handle_events(&event, flow);
            match event {
                Event::MainEventsCleared => {
                    let dbg: Option<&mut Debugger> = self.windows.debugger();
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

fn cycle(app: &mut App) {
}

fn main() {
    env_logger::init();

    let mut emu = app::Emulator::new();
    let app = App::new();
    let dbg = Debugger::new(emu.clone());
    app.create::<1280, 720, _, _>(Handle::Main, move |instance, win, event| EguiContext::new(instance, win, event, dbg))
        .run(move |app| {
            emu.cycle();
        });
}
