use super::*;

use egui::{SidePanel, CentralPanel, panel::Side, Style, TextStyle, Color32, Layout, Align, Vec2, Visuals, Rounding, FontFamily};
use egui::style::DebugOptions;

pub struct Debugger { }

impl Ui for Debugger {
    fn draw(&mut self, ctx: &egui::Context) {
        use egui::{FontId, TextStyle::*, FontFamily::Proportional};
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(18.0,FontFamily::Monospace)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style);
        CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
                egui::Frame::group(ui.style())
                    .fill(Color32::DARK_GRAY)
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.style_mut().visuals.override_text_color = Some(Color32::WHITE);
                        ui.columns(8, |uis: &mut [egui::Ui]| {
                            uis[0].with_layout(Layout::top_down(Align::Center), |ui: &mut egui::Ui| {
                                ui.label("A");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[1].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("F");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[2].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("B");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[3].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("C");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[4].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("D");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[5].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("E");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[6].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("H");
                                ui.label(format!("{:#x}", 0x12));
                            });
                            uis[7].with_layout(Layout::top_down(Align::Center),|ui: &mut egui::Ui| {
                                ui.label("L");
                                ui.label(format!("{:#x}", 0x12));
                            });
                        });
                    });
            });
    }
}

