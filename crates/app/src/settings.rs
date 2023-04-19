use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, WindowEvent};

use shared::{egui, Events};
use shared::egui::{Align, CentralPanel, Context, TextEdit, Ui};
use shared::input::KeyCat;
use shared::widgets::section::Section;
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
                    .with_tab(Tabs::Audio, audio::Audio::new(self, emu));
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("MODEL");
                });
                let model = &mut emu.cgb;
                ui.radio_value(model, Mode::Dmg, format!("{:?}", Mode::Dmg));
                ui.radio_value(model, Mode::Cgb, format!("{:?}", Mode::Cgb));
                ui.checkbox(&mut emu.bios, "enable boot rom");
                ui.separator();

                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label(if emu.link_do(|x| x.connected()) { "SERIAL - (Connected)" } else { "SERIAL" });
                });
                ui.label(format!("server listening on port {}", emu.link_port));

                ui.horizontal(|ui| {
                    let host = TextEdit::singleline(&mut emu.settings.host).desired_width(120.);
                    ui.label("Host: ");
                    ui.add(host);
                });
                ui.horizontal(|ui| {
                    let port = TextEdit::singleline(&mut emu.settings.port).desired_width(48.);
                    ui.label(" Port: ");
                    ui.add(port);
                });
                if ui.button("Connect").clicked() {
                    match (emu.settings.host.parse(), emu.settings.port.parse()) {
                        (Ok(addr), Ok(port)) => {
                            let addr: Ipv4Addr = addr;
                            let port: u16 = port;
                            emu.link_do(|link| link.connect(addr, port));
                        }
                        (a, p) => {
                            log::warn!("failed to parse: {a:?}, {p:?}");
                        }
                    }
                }
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("Gameplay");
                });
                ui.horizontal(|ui| {
                    ui.label("DMG palette: ");
                });
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
