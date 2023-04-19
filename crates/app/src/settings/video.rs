use shared::egui;
use shared::egui::{Align, Response, Ui, Widget};
use shared::utils::palette::Palette;
use crate::emulator::Emulator;

pub struct Video<'a> {
    emu: &'a mut Emulator
}

impl<'a> Video<'a> {
    pub fn new(emu: &'a mut Emulator) -> Self {
        Self {
            emu
        }
    }
}

impl<'a> Widget for Video<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.with_layout(egui::Layout::top_down(Align::Center), |ui| {
                ui.label("LCD - Colors");
            });
            let cmp = self.emu.settings.palette;
            let mut palette = cmp.to_f32();
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut palette[0]);
                ui.label("White");
            });
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut palette[1]);
                ui.label("Light Gray");
            });
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut palette[2]);
                ui.label("Dark Gray");
            });
            ui.horizontal(|ui| {
                ui.color_edit_button_rgb(&mut palette[3]);
                ui.label("Black");
            });
            let mut palette = Palette::from_f32(palette);
            ui.horizontal(|ui| {
                if ui.button("Default").clicked() {
                    palette = Palette::GrayScale;
                }
                if ui.button("Original").clicked() {
                    palette = Palette::Original;
                }
            });
            if cmp != palette {
                self.emu.settings.palette = palette;
                self.emu.console.bus.set_palette(&mut self.emu.console.gb, self.emu.settings.palette);
            }
            ui.separator();
        }).response
    }
}