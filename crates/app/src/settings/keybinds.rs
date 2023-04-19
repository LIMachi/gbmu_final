use winit::event::VirtualKeyCode;

use shared::egui;
use shared::egui::{Align, Button, Response, Ui, Vec2, Widget};
use shared::input::KeyCat;
use shared::widgets::section::Section;

use crate::emulator::Emulator;
use crate::settings::Settings;

struct Keybind<'a> {
    key: KeyCat,
    bind: Option<VirtualKeyCode>,
    value: &'a mut Option<KeyCat>,
}

impl<'a> Keybind<'a> {
    pub fn new(key: KeyCat, settings: &'a mut Settings, emu: &'a mut Emulator) -> Self {
        Self {
            bind: emu.bindings.has(key),
            key,
            value: &mut settings.key,
        }
    }
}

impl<'a> egui::Widget for Keybind<'a> {
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

struct KeybindSection<'s, I: IntoIterator<Item=KeyCat>> {
    settings: &'s mut Settings,
    emu: &'s mut Emulator,
    iter: I,
}

impl<'s, I: IntoIterator<Item=KeyCat>> KeybindSection<'s, I> {
    fn new(settings: &'s mut Settings, emu: &'s mut Emulator, iter: I) -> Self {
        Self {
            settings,
            emu,
            iter,
        }
    }
}

impl<'s, I: IntoIterator<Item=KeyCat>> egui::Widget for KeybindSection<'s, I> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for section in self.iter {
                ui.add(Keybind::new(section, self.settings, self.emu));
            }
        }).response
    }
}

pub struct Keybinds<'a> {
    settings: &'a mut Settings,
    emu: &'a mut Emulator,
}

impl<'a> Keybinds<'a> {
    pub fn new(settings: &'a mut Settings, emu: &'a mut Emulator) -> Self {
        Self { settings, emu }
    }
}

impl<'a> Widget for Keybinds<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let res = ui.section("JOYPAD", |ui| {
            ui.add(KeybindSection::new(self.settings, self.emu, KeyCat::joypad()))
        });
        res | ui.section("DEBUG", |ui| {
            ui.add(KeybindSection::new(self.settings, self.emu, KeyCat::shortcuts()))
        })
    }
}
