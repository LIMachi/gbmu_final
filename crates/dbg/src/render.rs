use super::{Emulator, Ninja, Disassembly};
use shared::{Ui, egui::{self, CentralPanel, Color32, Layout, Align, FontFamily, Widget, Response}, Break};
use shared::cpu::{Reg, Value, Opcode, Flags};
use shared::egui::{Frame, Id, LayerId, SidePanel, Stroke};
use crate::{Context, ImageLoader, Texture};

const DARK_BLACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x2A);

pub struct Register(&'static str, Value);

impl Widget for Register {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        ui.with_layout(Layout::top_down(Align::Center), |ui: &mut egui::Ui| {
            ui.label(self.0);
            ui.label(match self.1 {
                Value::U8(v) => format!("{:#04x}", v),
                Value::U16(v) => format!("{:#06x}", v),
            });
        }).response
    }
}

impl<E: Emulator> Ui for Ninja<E> {
    fn init(&mut self, ctx: &mut Context) {
        self.textures.insert(Texture::Play, ctx.load_svg::<40, 40>("play", "assets/icons/play.svg"));
        self.textures.insert(Texture::Pause, ctx.load_svg::<40, 40>("pause", "assets/icons/pause.svg"));
        self.textures.insert(Texture::Step, ctx.load_svg::<32, 32>("step", "assets/icons/step.svg"));
    }

    fn draw(&mut self, ctx: &egui::Context) {
        use egui::{FontId, TextStyle::*, FontFamily::Proportional};
        let mut style = (*ctx.style()).clone();
        style.visuals.override_text_color = Some(Color32::WHITE);
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(14.0,FontFamily::Monospace)),
            (Monospace, FontId::new(14.0, Proportional)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style.clone());
        let rect = ctx.available_rect();
        SidePanel::right("right")
            .show(ctx, |ui: &mut egui::Ui| {
                let w = rect.width() - 840.;
                ui.set_width(w);
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui: &mut egui::Ui| {
                    egui::Frame::group(ui.style())
                        .fill(DARK_BLACK)
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.spacing_mut().item_spacing.x = 16.;
                                let sz: egui::Vec2 = (32., 32.).into();
                                let pause = egui::ImageButton::new(self.tex(Texture::Pause), (40., 40.)).frame(false);
                                let reset = egui::ImageButton::new(self.tex(Texture::Pause), (40., 40.)).frame(false);

                                let play = egui::ImageButton::new(self.tex(Texture::Play), (40., 40.)).frame(false);
                                let step = egui::ImageButton::new(self.tex(Texture::Step), sz).frame(false);
                                if ui.add(reset).clicked() { self.emu.reset(); };
                                if ui.add(step).clicked() { self.emu.schedule_break(Break::Instructions(1)).play(); };
                                if ui.add(play).clicked() { self.emu.play(); };
                                if ui.add(pause.clone()).clicked() { self.emu.pause(); };
                            });
                        });
                });
            });
        CentralPanel::default()
            .show(ctx, |ui: &mut egui::Ui| {
                ui.vertical(|ui: &mut egui::Ui| {
                    ui.set_max_width(800.);
                    ui.spacing_mut().item_spacing.y = 0.;
                    egui::Frame::group(ui.style())
                        .fill(Color32::DARK_GREEN)
                        .show(ui, |ui: &mut egui::Ui| { ui.set_width(ui.available_width()) });
                    ui.spacing_mut().item_spacing.y = 24.;
                    egui::Frame::group(ui.style())
                        .fill(DARK_BLACK)
                        .stroke(Stroke::NONE)
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.spacing_mut().item_spacing.y = 0.;
                            ui.columns(11, |uis: &mut [egui::Ui]| {
                                let flags = self.emu.cpu_register(Reg::F);
                                uis[0].add(Register("A", self.emu.cpu_register(Reg::A)));
                                uis[1].add(Register("F", flags));
                                uis[2].add(Register("B", self.emu.cpu_register(Reg::B)));
                                uis[3].add(Register("C", self.emu.cpu_register(Reg::C)));
                                uis[4].add(Register("D", self.emu.cpu_register(Reg::D)));
                                uis[5].add(Register("E", self.emu.cpu_register(Reg::E)));
                                uis[6].add(Register("H", self.emu.cpu_register(Reg::H)));
                                uis[7].add(Register("L", self.emu.cpu_register(Reg::L)));
                                uis[8].add(Register("SP", self.emu.cpu_register(Reg::SP)));
                                uis[9].add(Register("PC", self.emu.cpu_register(Reg::PC)));
                                uis[10].with_layout(Layout::top_down(Align::Center), |ui: &mut egui::Ui| {
                                    ui.label("Flags");
                                    let f = flags.u8();
                                    ui.horizontal(|ui| {
                                        ui.label(if f.zero() { "Z" } else { "-" });
                                        ui.label(if f.sub() { "S" } else { "-" });
                                        ui.label(if f.half() { "H" } else { "-" });
                                        ui.label(if f.carry() { "C" } else { "-" });
                                    });
                                });
                            });
                        });
                    ui.spacing_mut().item_spacing.y = 0.;
                    egui::Frame::group(ui.style())
                        .fill(DARK_BLACK)
                        .show(ui, |ui: &mut egui::Ui| {
                            self.disassembly.render(&self.emu, ui);
                        });
                });
            });
    }
}

