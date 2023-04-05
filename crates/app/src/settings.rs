use std::borrow::{Borrow};
use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use apu::Controller;
use shared::{egui, Events};
use shared::egui::{Align, Button, CentralPanel, Context, Response, TextEdit, Ui, Vec2};
use shared::input::Section;
use crate::emulator::Emulator;

pub struct Settings {
    emu: Emulator,
    devices: Vec<String>,
    key: Option<Section>,
    host: String,
    port: String
}

impl Settings {
    pub fn new(emu: Emulator) -> Self {
        Self {
            devices: Controller::devices().collect(),
            emu,
            key: None,
            host: "127.0.0.1".to_string(),
            port: "27542".to_string()
        }
    }

}

// TODO mode auto ?
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum Mode {
    Dmg,
    Cgb
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

struct Keybind<'a> {
    key: Section,
    bind: Option<VirtualKeyCode>,
    value: &'a mut Option<Section>
}

impl<'a> Keybind<'a> {
    pub fn new(key: Section, settings: &'a mut Settings) -> Self {
        Self {
            bind: settings.emu.bindings.has(key),
            key,
            value: &mut settings.key
        }
    }
}

impl <'a>egui::Widget for Keybind<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label(format!("{:?}", self.key));
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                let button = Button::new(match (&self.value, self.bind) {
                    (Some(v), _) if v == &self.key => "...".to_string(),
                    (_, Some(keycode)) => format!("{keycode:?}"),
                    _ => "Unbound".to_string()
                }).min_size(Vec2::new(48., 0.));
                if ui.add(button).clicked() {
                    self.value.replace(self.key);
                }
            });
        }).response
    }
}

struct KeybindSection<'s, I: IntoIterator<Item = Section>> {
    settings: &'s mut Settings,
    iter: I
}

impl<'s, I: IntoIterator<Item = Section>> KeybindSection<'s, I> {
    fn new(settings: &'s mut Settings, iter: I) -> Self {
        Self {
            settings,
            iter
        }
    }
}

impl <'s, I: IntoIterator<Item = Section>>egui::Widget for KeybindSection<'s, I> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for section in self.iter {
                ui.add(Keybind::new(section, self.settings));
            }
        }).response
    }
}

impl shared::Ui for Settings {
    fn init(&mut self, _ctx: &mut Context) {
    }

    //TODO bouger les keybinds
    fn draw(&mut self, ctx: &mut Context) {
        CentralPanel::default()
            .show(ctx, |ui: &mut Ui| {
                let mut model = *self.emu.cgb.as_ref().borrow();
                let mut bios = *self.emu.bios.as_ref().borrow();
                let mut global_volume = *self.emu.audio_settings.volume.as_ref().borrow();
                let mut chan1 = *self.emu.audio_settings.channels[0].as_ref().borrow();
                let mut chan2 = *self.emu.audio_settings.channels[1].as_ref().borrow();
                let mut chan3 = *self.emu.audio_settings.channels[2].as_ref().borrow();
                let mut chan4 = *self.emu.audio_settings.channels[3].as_ref().borrow();
                let mut device = &self.emu.audio.device();

                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("JOYPAD");
                });
                ui.add(KeybindSection::new(self, Section::joypad()));
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("DEBUG");
                });
                ui.add(KeybindSection::new(self, Section::shortcuts()));
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("MODEL");
                });
                ui.radio_value(&mut model, Mode::Dmg, format!("{:?}", Mode::Dmg));
                ui.radio_value(&mut model, Mode::Cgb, format!("{:?}", Mode::Cgb));
                self.emu.cgb.replace(model);
                ui.checkbox(&mut bios, "enable boot rom");
                self.emu.bios.replace(bios);
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("SOUNDS");
                });
                ui.add(egui::Slider::new(&mut global_volume, 0f32..=1f32).text("Volume"));
                self.emu.audio_settings.volume.replace(global_volume);
                ui.checkbox(&mut chan1, "Channel 1 - Sweep");
                ui.checkbox(&mut chan2, "Channel 2 - Square");
                ui.checkbox(&mut chan3, "Channel 3 - Wave");
                ui.checkbox(&mut chan4, "Channel 4 - Noise");
                self.emu.audio_settings.channels[0].replace(chan1);
                self.emu.audio_settings.channels[1].replace(chan2);
                self.emu.audio_settings.channels[2].replace(chan3);
                self.emu.audio_settings.channels[3].replace(chan4);
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("AUDIO OUTPUT");
                });
                self.devices.iter().for_each(|dev| {
                    ui.radio_value(&mut device, dev, dev);
                });
                if *device != self.audio_device.device() {
                    self.audio_device.switch(device);
                }
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label(if self.emu.link_do(|x| x.connected()) { "SERIAL - (Connected)" } else { "SERIAL" });
                });
                ui.label(format!("server listening on port {}", self.emu.link_port));
                ui.horizontal(|ui| {
                    let host = TextEdit::singleline(&mut self.host).desired_width(120.);
                    ui.label("Host: ");
                    ui.add(host);
                });
                ui.horizontal(|ui| {
                    let port = TextEdit::singleline(&mut self.port).desired_width(48.);
                    ui.label(" Port: ");
                    ui.add(port);
                });
                if ui.button("Connect").clicked() {
                    match (self.host.parse(), self.port.parse()) {
                        (Ok(addr), Ok(port)) => {
                            let addr: Ipv4Addr = addr;
                            let port: u16 = port;
                            self.emu.link_do(|link| link.connect(addr, port));
                        },
                        (a, p) => {
                            log::warn!("failed to parse: {a:?}, {p:?}");
                        }
                    }
                }
            });
    }

    fn handle(&mut self, event: &Event<Events>) {
      match event {
          Event::WindowEvent { event: WindowEvent::KeyboardInput {
              input: KeyboardInput { virtual_keycode: Some(input), .. }, ..
          }, .. } => {
              if let Some(key) = self.key.take() {
                  self.emu.bindings.set(key,*input);
              }
          }
          _ => {}
      }
    }
}
