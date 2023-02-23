use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use shared::egui;
use shared::egui::{Color32, Margin, Response, SelectableLabel, Ui, Widget};

pub trait Tab {
    fn name(&self) -> String;
}

pub struct Tabs<'a, E: PartialEq + Eq + Clone + Hash + Tab> {
    current: &'a mut E,
    select: Vec<E>,
    tabs: HashMap<E, Box<dyn FnOnce(&mut Ui) -> Response + 'a>>
}

impl<'a, E: PartialEq + Eq + Clone + Hash + Tab> Widget for Tabs<'a, E> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let current = self.tabs.remove(self.current).unwrap();
        egui::Frame::group(ui.style())
            .fill(super::DARK_BLACK)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for name in self.select {
                        ui.spacing_mut().button_padding.x = 8.;
                        ui.spacing_mut().button_padding.y = 8.;
                        if ui.selectable_label(&name == self.current, name.name())
                            .clicked() {
                            *self.current = name;
                        }
                    }
                });
            });
        egui::Frame::none()
            .outer_margin(Margin::symmetric(0., 8.))
            .show(ui, current).response
    }
}

impl<'a, E: PartialEq + Eq + Hash + Clone + Tab> Tabs<'a, E> {
    pub fn new(current: &'a mut E) -> Self {
        Self {
            current,
            select: Vec::with_capacity(8),
            tabs: HashMap::with_capacity(8)
        }
    }

    pub fn with_tab<W: Widget, F: FnOnce() -> W + 'a>(mut self, name: E, tab: F) -> Self {
        self.select.push(name.clone());
        self.tabs.insert(name, Box::new(|ui| { ui.add(tab()) }));
        self
    }
}
