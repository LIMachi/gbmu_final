use std::cell::RefCell;
use std::rc::Rc;
use winit::{
    event_loop::{ControlFlow, EventLoopWindowTarget}
};
use winit::event_loop::EventLoopBuilder;

mod log;
mod render;
mod emulator;
pub mod app;

use app::Menu;
use dbg::Debugger;
use render::{windows::Windows, WindowType};
use shared::breakpoints::Breakpoints;
use shared::{Events, Handle};
use shared::utils::Cell;
use shared::utils::clock::{Chrono, Clock};
use crate::emulator::Keybindings;
use crate::render::{Event, EventLoop, Proxy};

pub struct App {
    bindings: Rc<RefCell<emulator::Keybindings>>,
    emu: emulator::Emulator,
    dbg: Debugger<emulator::Emulator>,
    event_loop: Option<EventLoop>,
    pub windows: Windows
}

impl App {
    pub fn new() -> Self {
        let e = EventLoopBuilder::with_user_event()
            .build();
        let proxy = e.create_proxy();
        let bindings = Keybindings::default().cell();
        let mut emu = emulator::Emulator::new(
            proxy.clone(),
            bindings.clone(),
            Breakpoints::default());
        let dbg = Debugger::new(emu.clone());
        Self {
            bindings,
            event_loop: Some(e),
            windows: Windows::new(proxy),
            emu,
            dbg
        }
    }

    pub fn proxy(&self) -> Proxy { self.event_loop.as_ref().unwrap().create_proxy() }

    pub fn open(&mut self, handle: WindowType, event_loop: &EventLoopWindowTarget<Events>) -> &mut Self {
        self.windows.create(handle, event_loop);
        self
    }

    pub fn create(mut self, handle: WindowType) -> Self {
        self.windows.create(handle, self.event_loop.as_ref().unwrap());
        self
    }

    pub fn handle_events(&mut self, event: &Event, target: &EventLoopWindowTarget<Events>, flow: &mut ControlFlow) {
        match event {
            Event::UserEvent(Events::Play(_)) => {
                if !self.windows.is_open(Handle::Game) {
                    self.open(WindowType::Game(self.emu.clone()), target);
                }
            },
            Event::UserEvent(Events::Open(handle)) => {
              self.open(match handle {
                  Handle::Main => unimplemented!(),
                  Handle::Debug => WindowType::Debug(self.dbg.clone()),
                  Handle::Game => WindowType::Game(self.emu.clone()),
                  Handle::Sprites => WindowType::Sprites(self.emu.clone()),
                  Handle::Settings => unimplemented!()
              }, target);
            },
            _ => {}
        }
        self.windows.handle_events(event, flow);
    }

    pub fn run<F: 'static + FnMut(&mut App)>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        event.run(move |mut event: Event, target: &EventLoopWindowTarget<Events>, flow: &mut ControlFlow| {
            flow.set_poll();
            self.handle_events(&event, target, flow);
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
    log::init();
    let mut app = App::new();
    let menu = WindowType::Main(Menu::new(app.proxy()));
    let mut st = Chrono::new();
    let mut current = std::time::Instant::now();
    let mut s = 0;
    let mut acc = 0.0;
    let mut cycles = 0;
    let mut clock = Clock::new(4);
    app.create(menu)
        .run(move |app| {
            if app.emu.is_running() {
                if st.paused() {
                    current = std::time::Instant::now();
                    st.start();
                }
                acc += current.elapsed().as_secs_f64();
                current = std::time::Instant::now();
                let cy = app.emu.cycle_time();
                while acc >= cy {
                    cycles += 1;
                    clock.tick();
                    if !app.emu.cycle(clock.value()) {
                        st.pause();
                        break ;
                    }
                    acc -= cy;
                }
            } else {
                st.pause();
            }
            if s != st.elapsed().as_secs() {
                s = st.elapsed().as_secs();
                log::debug!("cycles: {}", cycles as f64 / st.elapsed().as_secs_f64());
            }
        });
}
