use std::net::Ipv4Addr;
use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use shared::{egui, Events};
use shared::egui::{Align, Button, CentralPanel, Context, Response, TextEdit, Ui, Vec2};
use shared::input::Section;
use crate::emulator::Emulator;

pub struct Settings {
    devices: Vec<String>,
    key: Option<Section>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            devices: apu::Controller::devices().collect(),
            key: None
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
    pub fn new(key: Section, settings: &'a mut Settings, emu: &'a mut Emulator) -> Self {
        Self {
            bind: emu.bindings.has(key),
            key,
            value: &mut settings.key,
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
    emu: &'s mut Emulator,
    iter: I
}

impl<'s, I: IntoIterator<Item = Section>> KeybindSection<'s, I> {
    fn new(settings: &'s mut Settings, emu: &'s mut Emulator, iter: I) -> Self {
        Self {
            settings,
            emu,
            iter
        }
    }
}

impl <'s, I: IntoIterator<Item = Section>>egui::Widget for KeybindSection<'s, I> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for section in self.iter {
                ui.add(Keybind::new(section, self.settings, self.emu));
            }
        }).response
    }
}

impl shared::Ui for Settings {
    type Ext = Emulator;

    fn init(&mut self, _ctx: &mut Context, _ext: &mut Emulator) {}

    //TODO bouger les keybinds
    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        CentralPanel::default()
            .show(ctx, |ui: &mut Ui| {
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("JOYPAD");
                });
                ui.add(KeybindSection::new(self, emu, Section::joypad()));
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("DEBUG");
                });
                ui.add(KeybindSection::new(self, emu,Section::shortcuts()));
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("MODEL");
                });
                let model = &mut emu.cgb;
                ui.radio_value(model, Mode::Dmg, format!("{:?}", Mode::Dmg));
                ui.radio_value(model, Mode::Cgb, format!("{:?}", Mode::Cgb));
                ui.checkbox( &mut emu.bios, "enable boot rom");
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("SOUNDS");
                });
                ui.add(egui::Slider::new(&mut emu.audio_settings.volume, 0f32..=1f32).text("Volume"));
                ui.checkbox(&mut emu.audio_settings.channels[0], "Channel 1 - Sweep");
                ui.checkbox(&mut emu.audio_settings.channels[1], "Channel 2 - Square");
                ui.checkbox(&mut emu.audio_settings.channels[2], "Channel 3 - Wave");
                ui.checkbox(&mut emu.audio_settings.channels[3], "Channel 4 - Noise");
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("AUDIO OUTPUT");
                });
                let mut device = &emu.audio.device();
                let devices: Vec<&String> = self.devices.iter().collect();
                devices.iter().for_each(|dev| {
                    ui.radio_value(&mut device, dev, *dev);
                });
                if device != &emu.audio.device() {
                    let device = device.clone().clone();
                    emu.audio.switch(device, &mut emu.console.gb.apu);
                }
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
                        },
                        (a, p) => {
                            log::warn!("failed to parse: {a:?}, {p:?}");
                        }
                    }
                }

            });
    }

    fn handle(&mut self, event: &Event<Events>, emu: &mut Emulator) {
      match event {
          Event::WindowEvent { event: WindowEvent::KeyboardInput {
              input: KeyboardInput { virtual_keycode: Some(input), .. }, ..
          }, .. } => {
              if let Some(key) = self.key.take() {
                  emu.bindings.set(key,*input);
              }
          }
          _ => {}
      }
    }
}
