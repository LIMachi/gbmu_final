use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, WindowEvent};

use shared::Events;
use shared::egui::{CentralPanel, Context, Ui};
use shared::input::KeyCat;
use shared::widgets::tabs;
use shared::widgets::tabs::Tab;

use crate::emulator::Emulator;

mod keybinds;
mod audio;
mod device;
mod video;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Tabs {
    Keybinds,
    Device,
    Audio,
    Video,
}

impl Tab for Tabs {
    fn name(&self) -> String {
        format!("{:?}", self)
    }
}

pub struct Settings {
    tab: Tabs,
    devices: Vec<String>,
    key: Option<KeyCat>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tab: Tabs::Keybinds,
            devices: apu::Controller::devices().collect(),
            key: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Mode {
    Dmg,
    Cgb,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Dmg
    }
}

impl Mode {
    pub fn is_cgb(&self) -> bool {
        match self {
            Mode::Dmg => false,
            Mode::Cgb => true
        }
    }
}

impl shared::Ui for Settings {
    type Ext = Emulator;

    fn init(&mut self, _ctx: &mut Context, _ext: &mut Emulator) {}

    //TODO bouger les keybinds
    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        CentralPanel::default()
            .show(ctx, |ui: &mut Ui| {
                tabs::Tabs::new(&mut self.tab, ui, &[Tabs::Keybinds, Tabs::Device, Tabs::Audio, Tabs::Video])
                    .with_tab(Tabs::Keybinds, keybinds::Keybinds::new(self, emu))
                    .with_tab(Tabs::Audio, audio::Audio::new(self, emu))
                    .with_tab(Tabs::Device, device::Device::new(emu))
                    .with_tab(Tabs::Video, video::Video::new(emu));
            });
    }

    fn handle(&mut self, event: &Event<Events>, emu: &mut Emulator) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { virtual_keycode: Some(input), .. }, ..
                }, ..
            } => {
                if let Some(key) = self.key.take() {
                    emu.bindings.set(key, *input);
                }
            }
            _ => {}
        }
    }
}
