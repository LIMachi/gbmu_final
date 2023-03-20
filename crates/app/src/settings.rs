use std::cell::RefCell;
use std::rc::Rc;
use serde::{Deserialize, Serialize};
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use shared::{egui, Events};
use shared::egui::{Align, Button, CentralPanel, Context, Response, Ui, Vec2};
use shared::input::{Keybindings, Section};

pub struct Settings {
    bindings: Keybindings,
    cgb: Rc<RefCell<Mode>>,
    key: Option<Section>
}

impl Settings {
    pub fn new(bindings: Keybindings, cgb: Rc<RefCell<Mode>>) -> Self {
        Self {
            bindings,
            cgb,
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
                let mut value = *self.cgb.as_ref().borrow();
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
                ui.radio_value(&mut value, Mode::Dmg, format!("{:?}", Mode::Dmg));
                ui.radio_value(&mut value, Mode::Cgb, format!("{:?}", Mode::Cgb));
                self.cgb.replace(value);
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
