#![feature(hash_drain_filter)]
#![feature(is_some_and)]

extern crate core;

use std::time::Duration;

use winit::{
    event_loop::{ControlFlow, EventLoopWindowTarget}
};
use winit::event_loop::EventLoopBuilder;

use render::windows::Windows;
use shared::{Events, Handle};
use shared::utils::clock::Chrono;

use crate::app::{AppConfig, DbgConfig};
use crate::render::{Event, EventLoop, Proxy};

mod log;
mod render;
mod settings;
mod emulator;
pub mod app;

pub struct App {
    emu: emulator::Emulator,
    event_loop: Option<EventLoop>,
    pub windows: Windows,
}

impl App {
    pub fn new() -> Self {
        let e = EventLoopBuilder::with_user_event()
            .build();
        let proxy = e.create_proxy();
        let conf = AppConfig::load();
        let emu = emulator::Emulator::new(proxy.clone(), conf);
        Self {
            event_loop: Some(e),
            windows: Windows::new(proxy),
            emu,
        }
    }

    pub fn proxy(&self) -> Proxy { self.event_loop.as_ref().unwrap().create_proxy() }

    pub fn open<'a>(&mut self, handle: Handle, event_loop: &EventLoopWindowTarget<Events>) -> &mut Self {
        self.windows.create(handle, &mut self.emu, event_loop);
        self
    }

    pub fn create(mut self, handle: Handle) -> Self {
        self.windows.create(handle, &mut self.emu, self.event_loop.as_ref().unwrap());
        self
    }

    pub fn handle_events(&mut self, event: &Event, target: &EventLoopWindowTarget<Events>, flow: &mut ControlFlow) {
        match event {
            Event::UserEvent(Events::Play(_)) => {
                if !self.windows.is_open(Handle::Game) {
                    self.open(Handle::Game, target);
                }
            }
            Event::UserEvent(Events::Open(handle)) => {
                let handle = *handle;
                if !self.windows.is_open(handle) {
                    self.open(handle, target);
                }
            }
            Event::UserEvent(Events::Close(handle)) => {
                self.windows.close(*handle);
            }
            _ => {}
        }
        self.windows.handle_events(event, flow, &mut self.emu);
    }

    pub fn run<F: 'static + FnMut(&mut App)>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        event.run(move |event: Event, target: &EventLoopWindowTarget<Events>, flow: &mut ControlFlow| {
            flow.set_poll();
            self.handle_events(&event, target, flow);
            match event {
                Event::MainEventsCleared => {
                    handler(&mut self);
                    self.windows.update(&mut self.emu);
                }
                Event::UserEvent(Events::Quit) => {
                    self.emu.stop(true);
                    let conf = AppConfig {
                        sound_device: self.emu.audio.config(),
                        audio_settings: self.emu.audio_settings.clone(),
                        roms: self.emu.roms.clone(),
                        debug: DbgConfig {
                            breaks: self.emu.breakpoints.take().into_iter()
                                .filter(|x| !x.temp())
                                .collect(),
                            and: self.emu.breakpoints.and()
                        },
                        emu: self.emu.settings.clone(),
                        keys: self.emu.bindings.clone(),
                        mode: self.emu.mode(),
                        bios: self.emu.enabled_boot(),
                    };
                    if let Err(e) = serde_any::ser::to_file_pretty("gbmu.ron", &conf) {
                        log::warn!("error while saving config {e:?}");
                    }
                    flow.set_exit();
                }
                _ => {}
            }
        })
    }
}

fn main() {
    log::init();
    let app = App::new();
    let mut st = Chrono::new();
    let mut current = std::time::Instant::now();
    let mut acc = 0.0;
    let mut cycles = 0;
    let mut run = false;
    let mut dt = Duration::from_secs(0);
    app.create(Handle::Main)
        .run(move |app| {
            if app.emu.is_running() {
                if st.paused() { st.start(); }
                if run {
                    acc += current.elapsed().as_secs_f64();
                    current = std::time::Instant::now();
                    let cy = app.emu.cycle_time();
                    while acc >= cy {
                        cycles += 1;
                        app.emu.cycle();
                        acc -= cy;
                    }
                    dt += current.elapsed();
                } else if st.elapsed().as_secs_f64() > 0.1 {
                    run = true;
                    current = std::time::Instant::now();
                    st.restart();
                }
                if st.elapsed().as_secs() != 0 {
                    let t = cycles as f64 / st.elapsed().as_secs_f64();
                    st.restart();
                    let p = (t / 4194304.) * 100.;
                    let n = dt.as_secs_f64() * 100.;
                    log::debug!("cycles: {:.0} ({:0.2} %) | took {dt:?} ({n:0.2}% capacity)", t, p);
                    dt = Duration::from_secs(0);
                    cycles = 0;
                }
            } else {
                st.stop();
                cycles = 0;
                run = false;
                acc = 0.;
            }
        });
}
