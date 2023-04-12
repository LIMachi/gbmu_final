use std::collections::HashMap;

use std::hash::Hash;
use shared::egui;
use shared::egui::{Margin, Response, Sense, Ui, Vec2, Widget};

pub trait Tab {
    fn name(&self) -> String;
}

pub struct Tabs<'a, 'ui, E: PartialEq + Eq + Clone + Hash + Tab> {
    current: &'a mut E,
    ui: &'ui mut egui::Ui,
    res: Option<Response>
}

impl<'a, 'ui, E: PartialEq + Eq + Hash + Clone + Tab> Tabs<'a, 'ui, E> {
    pub fn new(current: &'a mut E, ui: &'ui mut Ui, values: &[E]) -> Self {
        egui::Frame::group(ui.style())
            .fill(super::DARK_BLACK)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for name in values {
                        ui.spacing_mut().button_padding.x = 8.;
                        ui.spacing_mut().button_padding.y = 8.;
                        if ui.selectable_label(name == current, name.name())
                            .clicked() {
                            *current = name.clone();
                        }
                    }
                });
            });
        Self { current, ui, res: None }
    }

    pub fn with_tab<W: Widget>(mut self, name: E, tab: W) -> Self {
        if &name == self.current {
            self.res = Some(egui::Frame::none()
                .outer_margin(Margin::symmetric(0., 8.))
                .show(self.ui, |ui| ui.add(tab)).response);
        }
        self
    }

    pub fn response(mut self) -> Response {
        self.res.take().unwrap_or_else(|| {
            log::warn!("no tab matched current ! Is a tab missing ?");
            self.ui.allocate_response(Vec2::new(0., 0.), Sense::hover())
        })
    }
}
