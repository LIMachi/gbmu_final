use super::{Emulator, Ninja};
use shared::{Ui, egui::{self, CentralPanel, Color32, Layout, Align, FontFamily, Widget, Response}};
use shared::cpu::{Reg, Value, Opcode};

pub struct Register(&'static str, Value);

impl Widget for Register {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.with_layout(Layout::top_down(Align::Center), |ui: &mut egui::Ui| {
            ui.label(self.0);
            ui.label(format!("{:#x}", self.1));
        }).response
    }
}

impl<E: Emulator> Ui for Ninja<E> {
    fn draw(&mut self, ctx: &egui::Context) {
        use egui::{FontId, TextStyle::*, FontFamily::Proportional};
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(16.0,FontFamily::Monospace)), //because outline text
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style);
        CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let frame = egui::Frame {
                stroke: egui::Stroke {
                    width: 2.,
                    color: egui::Color32::BLACK,
                },
                ..egui::Frame::default()
            };
                //.fill(Color32::GRAY)
                frame.show(ui, |ui: &mut egui::Ui| {
                    ui.style_mut().visuals.override_text_color = Some(Color32::GREEN);
                    ui.columns(10, |uis: &mut [egui::Ui]| {
                        uis[0].add(Register("A", self.emu.cpu_register(Reg::A)));
                        uis[1].add(Register("F", self.emu.cpu_register(Reg::F)));
                        uis[2].add(Register("B", self.emu.cpu_register(Reg::B)));
                        uis[3].add(Register("C", self.emu.cpu_register(Reg::C)));
                        uis[4].add(Register("D", self.emu.cpu_register(Reg::D)));
                        uis[5].add(Register("E", self.emu.cpu_register(Reg::E)));
                        uis[6].add(Register("H", self.emu.cpu_register(Reg::H)));
                        uis[7].add(Register("L", self.emu.cpu_register(Reg::L)));
                        uis[8].add(Register("SP", self.emu.cpu_register(Reg::SP)));
                        uis[9].add(Register("PC", self.emu.cpu_register(Reg::PC)));
                    });
                });
        });
    }
}

