#![feature(hash_drain_filter)]

use std::cell::RefCell;
use std::rc::Rc;
use winit::{
    event_loop::{ControlFlow, EventLoopWindowTarget}
};
use winit::event_loop::EventLoopBuilder;

mod log;
mod render;
mod settings;
mod emulator;
pub mod app;

use app::Menu;
use apu::SoundConfig;
use dbg::{Ninja, Schedule};
use render::{windows::Windows, WindowType};
use shared::{Events, Handle};
use shared::audio_settings::AudioSettings;
use shared::input::Keybindings;
use shared::utils::Cell;
use shared::utils::clock::{Chrono, Clock};
use crate::app::{AppConfig, DbgConfig, RomConfig};
use crate::render::{Event, EventLoop, Proxy};

pub struct App {
    sound_device: SoundConfig,
    audio_settings: AudioSettings,
    bindings: Keybindings,
    roms: Rc<RefCell<RomConfig>>,
    emu: emulator::Emulator,
    event_loop: Option<EventLoop>,
    pub windows: Windows
}

impl App {
    pub fn new() -> Self {
        let e = EventLoopBuilder::with_user_event()
            .build();
        let proxy = e.create_proxy();
        let conf = AppConfig::load();
        let bindings = conf.keys.clone();
        let emu = emulator::Emulator::new(proxy.clone(), bindings.clone(), &conf);
        let roms = conf.roms.cell();
        Self {
            sound_device: conf.sound_device,
            audio_settings: conf.audio_settings,
            roms,
            bindings,
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

    pub fn menu(&self) -> Menu {
        Menu::new(self.roms.clone(), self.proxy())
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
            },
            Event::UserEvent(Events::Open(handle)) => {
                let handle = *handle;
                if !self.windows.is_open(handle) {
                    self.open(handle, target);
                }
            },
            _ => {}
        }
        self.windows.handle_events(event, flow);
    }

    pub fn run<F: 'static + FnMut(&mut App)>(mut self, mut handler: F) -> ! {
        let event = self.event_loop.take().expect("yeah no");
        event.run(move |event: Event, target: &EventLoopWindowTarget<Events>, flow: &mut ControlFlow| {
            flow.set_poll();
            self.handle_events(&event, target, flow);
            match event {
                Event::MainEventsCleared => {
                    handler(&mut self);
                    self.windows.update();
                },
                Event::UserEvent(Events::Close) => {
                    let conf = AppConfig {
                        sound_device: self.sound_device.clone(),
                        audio_settings: self.audio_settings.clone(),
                        roms: self.roms.as_ref().take(),
                        debug: DbgConfig {
                            breaks: self.emu.breakpoints().take().into_iter()
                                .filter(|x| !x.temp())
                                .collect()
                        },
                        emu: self.emu.settings.take(),
                        keys: self.bindings.clone(),
                        mode: self.emu.mode(),
                        bios: self.emu.enabled_boot(),
                    };
                    if let Err(e) = serde_any::ser::to_file_pretty("gbmu.ron", &conf) {
                        log::warn!("error while saving config {e:?}");
                    }
                    flow.set_exit();
                }
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
    let app = App::new();
    let menu = WindowType::Main(app.menu());
    let mut st = Chrono::new();
    let mut current = std::time::Instant::now();
    let mut acc = 0.0;
    let mut cycles = 0;
    let mut clock = Clock::new(4);
    let mut run = false;
    app.create(menu)
        .run(move |app| {
            if app.emu.is_running() {
                if st.paused() { st.start(); }
                if run {
                    acc += current.elapsed().as_secs_f64();
                    current = std::time::Instant::now();
                    let cy = app.emu.cycle_time();
                    while acc >= cy {
                        cycles += 1;
                        app.emu.cycle(clock.tick());
                        acc -= cy;
                    }
                }
                if st.elapsed().as_secs() != 0 {
                    if !run {
                        run = true;
                        current = std::time::Instant::now();
                    }
                    let t = cycles as f64 / st.elapsed().as_secs_f64();
                    let p = (t / 4194304.) * 100.;
                    log::debug!("cycles: {:.0} ({:0.2} %)", t, p);
                    st.stop();
                    st.start();
                    cycles = 0;
                }
            } else {
                st.stop();
                cycles = 0;
                run = false;
            }
        });
}
