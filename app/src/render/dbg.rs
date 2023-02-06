use super::*;

use egui::{SidePanel, CentralPanel, panel::Side, Style, TextStyle, Color32, Layout, Align, Vec2, Visuals};
use egui::style::DebugOptions;

pub struct Debugger { }

impl Ui for Debugger {
    fn draw(&mut self, ctx: &egui::Context) {
        ctx.set_style(Style {
            debug: DebugOptions { debug_on_hover: true, ..Default::default() },
            visuals: Visuals {
                override_text_color: Some(Color32::WHITE),
                ..Default::default()
            },
            ..Default::default()
        });
        CentralPanel::default()
            .show(ctx, |ui: &mut egui::Ui| {
                let rect = ui.max_rect();
                ui.allocate_ui_with_layout(Vec2::new(rect.width(), rect.height()), Layout::top_down(Align::LEFT), |ui: &mut egui::Ui| {
                    ui.columns(8, |uis: &mut [egui::Ui]| {
                        uis[0].set_max_width(80.);
                        uis[1].set_max_width(80.);
                        uis[2].set_max_width(80.);
                        uis[3].set_max_width(80.);
                        uis[4].set_max_width(80.);
                        uis[5].set_max_width(80.);
                        uis[6].set_max_width(80.);
                        uis[7].set_max_width(80.);
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
                        })
                });
            });
    }
}
