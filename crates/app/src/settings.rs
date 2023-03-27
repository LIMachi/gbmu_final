use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use apu::Controller;
use shared::{egui, Events};
use shared::audio_settings::AudioSettings;
use shared::egui::{Align, Button, CentralPanel, Context, Response, TextBuffer, Ui, Vec2};
use shared::input::{Keybindings, Section};

pub struct Settings {
    bindings: Keybindings,
    cgb: Rc<RefCell<Mode>>,
    audio: AudioSettings,
    audio_device: Controller,
    devices: Vec<String>,
    key: Option<Section>
}

impl Settings {
    pub fn new(bindings: Keybindings, cgb: Rc<RefCell<Mode>>, audio: AudioSettings, audio_device: Controller) -> Self {
        Self {
            bindings,
            cgb,
            audio,
            audio_device,
            devices: Controller::devices().collect(),
            key:None
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
            bind: settings.bindings.has(key),
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
                let mut model = *self.cgb.as_ref().borrow();
                let mut global_volume = *self.audio.volume.as_ref().borrow();
                let mut chan1 = *self.audio.channels[0].as_ref().borrow();
                let mut chan2 = *self.audio.channels[1].as_ref().borrow();
                let mut chan3 = *self.audio.channels[2].as_ref().borrow();
                let mut chan4 = *self.audio.channels[3].as_ref().borrow();
                let mut device = self.audio_device.device();
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
                self.cgb.replace(model);
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("SOUNDS");
                });
                ui.add(egui::Slider::new(&mut global_volume, 0f32..=1f32).text("Volume"));
                self.audio.volume.replace(global_volume);
                ui.checkbox(&mut chan1, "Channel 1 - Sweep");
                ui.checkbox(&mut chan2, "Channel 2 - Square");
                ui.checkbox(&mut chan3, "Channel 3 - Wave");
                ui.checkbox(&mut chan4, "Channel 4 - Noise");
                self.audio.channels[0].replace(chan1);
                self.audio.channels[1].replace(chan2);
                self.audio.channels[2].replace(chan3);
                self.audio.channels[3].replace(chan4);
                ui.separator();
                ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                    ui.label("AUDIO OUTPUT");
                });
                self.devices.iter().for_each(|dev| {
                    ui.radio_value(&mut device, dev.clone(), dev);
                });
                if device != self.audio_device.device() {
                    self.audio_device.switch(device);
                }
            });
    }

    fn handle(&mut self, event: &Event<Events>) {
      match event {
          Event::WindowEvent { event: WindowEvent::KeyboardInput {
              input: KeyboardInput { virtual_keycode: Some(input), .. }, ..
          }, .. } => {
              if let Some(key) = self.key.take() {
                  self.bindings.set(key,*input);
              }
          }
          _ => {}
      }
    }
}
