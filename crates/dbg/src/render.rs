use std::str::FromStr;
use egui_extras::Column;
use super::{Emulator, Ninja, Disassembly};
use shared::{Ui, egui::{self, CentralPanel, Color32, Layout, Align, FontFamily, Widget, Response}};
use shared::breakpoints::{Breakpoint};
use shared::cpu::{Reg, Value, Flags};
use shared::egui::{ScrollArea, SidePanel, Stroke};
use shared::io::IO;
use shared::utils::image::ImageLoader;
use crate::{Bus, Context, Texture};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Reg(Reg),
    Cycles,
    Instructions
}

impl Kind {
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Reg(r) => r.name(),
            Kind::Cycles => "Cycles",
            Kind::Instructions => "Ins",
        }
    }
}

pub struct Data {
    raw: String,
    value: Value,
    reg: Kind,
    count: usize
}

trait Converter {
    fn convert(str: &str) -> Self;
}

impl Converter for u8 {
    fn convert(raw: &str) -> Self {
        let str = raw.trim_start_matches("0x").trim_start_matches("0X");
        if str != raw {
            u8::from_str_radix(str, 16).unwrap_or(0)
        } else {
            u8::from_str(str).unwrap_or(0)
        }
    }
}

impl Converter for u16 {
    fn convert(raw: &str) -> Self {
        let str = raw.trim_start_matches("0x").trim_start_matches("0X");
        if str != raw {
            u16::from_str_radix(str, 16).unwrap_or(0)
        } else {
            u16::from_str(str).unwrap_or(0)
        }
    }
}
impl Data {
    fn breakpoint(&self) -> Breakpoint {
        match self.reg {
            Kind::Reg(reg) =>  Breakpoint::register(reg, self.value),
            Kind::Cycles => Breakpoint::cycles(self.count),
            Kind::Instructions => Breakpoint::instructions(self.count),
        }


    }

    fn parse(&mut self) {
        match (self.value, self.reg) {
            (_, Kind::Cycles | Kind::Instructions) => { self.count = usize::from_str(&self.raw).unwrap_or(0); }
            (Value::U8(v), _) => { self.value = Value::U8(u8::convert(&self.raw)); },
            (Value::U16(v), _) => { self.value = Value::U16(u16::convert(&self.raw)); },
        };
    }

    fn update(&mut self) {
        match (self.reg, self.value) {
            (Kind::Reg(r), Value::U16(v)) if r == Reg::A || r == Reg::B || r == Reg::C || r == Reg::D || r == Reg::E || r == Reg::F || r == Reg::H || r == Reg::L => { self.value = Value::U8(v as u8); },
            (Kind::Reg(r), Value::U8(v)) if r == Reg::AF || r == Reg::BC || r == Reg::DE || r == Reg::HL || r == Reg::PC || r == Reg::SP => { self.value = Value::U16(v as u16); },
            _ => {}
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self { reg: Kind::Reg(Reg::PC), count: 0, raw: "".to_string(), value: Value::U16(0) }
    }
}

pub const DARK_BLACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x2A);

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

fn io_table(ui: &mut egui::Ui, ios: &[IO], bus: &&dyn Bus, source: &'static str) {
    ScrollArea::vertical()
        .id_source("ScrollArea_".to_owned() + source)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.push_id("IOTable_".to_owned() + source, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::exact(100.), 2)
                    .auto_shrink([false, false])
                    .body(|mut body| {
                        for io in ios {
                            body.row(16., |mut row| {
                                row.col(|ui| { ui.label(io.name()); });
                                row.col(|ui| { ui.label(format!("{:#04X}", bus.read(*io as u16))); });
                            })
                        }
                    });
            });
        });
}

impl<E: Emulator> Ui for Ninja<E> {
    fn init(&mut self, ctx: &mut Context) {
        self.textures.insert(Texture::Play, ctx.load_svg::<40, 40>("play", "assets/icons/play.svg"));
        self.textures.insert(Texture::Pause, ctx.load_svg::<40, 40>("pause", "assets/icons/pause.svg"));
        self.textures.insert(Texture::Step, ctx.load_svg::<32, 32>("step", "assets/icons/step.svg"));
        self.textures.insert(Texture::Reset, ctx.load_svg::<40, 40>("reset", "assets/icons/reset.svg"));
    }

    fn draw(&mut self, ctx: &egui::Context) {
        use egui::{FontId, TextStyle::*, FontFamily::Proportional};
        let mut style = (*ctx.style()).clone();
        style.visuals.override_text_color = Some(Color32::WHITE);
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(12.0,FontFamily::Monospace)),
            (Monospace, FontId::new(10.0, FontFamily::Monospace)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style.clone());
        let rect = ctx.available_rect();
        SidePanel::left("left")
            .show(ctx, |ui| {
                let emu = self.emu.bus();
                let bus = emu.bus();
                self.viewer.render(ui, &bus);
            });
        SidePanel::right("right")
            .show(ctx, |ui: &mut egui::Ui| {
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui: &mut egui::Ui| {
                    egui::Frame::group(ui.style())
                        .fill(DARK_BLACK)
                        .show(ui, |ui: &mut egui::Ui| {
                            ui.horizontal(|ui: &mut egui::Ui| {
                                ui.spacing_mut().item_spacing.x = 16.;
                                let sz: egui::Vec2 = (32., 32.).into();
                                let pause = egui::ImageButton::new(self.tex(Texture::Pause), (40., 40.)).frame(false);
                                let reset = egui::ImageButton::new(self.tex(Texture::Reset), (40., 40.)).frame(false);

                                let play = egui::ImageButton::new(self.tex(Texture::Play), (40., 40.)).frame(false);
                                let step = egui::ImageButton::new(self.tex(Texture::Step), sz).frame(false);
                                if ui.add(reset).clicked() { self.emu.reset(); };
                                if ui.add(step).clicked() { self.step() };
                                if ui.add(play).clicked() { self.emu.play(); };
                                if ui.add(pause.clone()).clicked() { self.pause(); };
                            });
                        });
                });
                egui::Frame::group(ui.style())
                    .show(ui, |ui: &mut egui::Ui| {
                        ui.horizontal(|ui| {
                            ui.menu_button(self.render_data.reg.name(), |ui| {
                                use Reg::*;
                                for r in [Kind::Reg(A), Kind::Reg(B), Kind::Reg(C), Kind::Reg(D), Kind::Reg(E), Kind::Reg(H), Kind::Reg(L), Kind::Reg(AF), Kind::Reg(BC), Kind::Reg(DE), Kind::Reg(HL), Kind::Reg(SP), Kind::Reg(PC), Kind::Cycles, Kind::Instructions] {
                                    if ui.selectable_value(&mut self.render_data.reg, r, r.name()).clicked() {
                                        self.render_data.update();
                                        ui.close_menu();
                                    }
                                }
                            });
                            if ui.add(egui::TextEdit::singleline(&mut self.render_data.raw).desired_width(64.)).changed() {
                                self.render_data.parse();
                            }
                            if ui.button("BREAK").clicked() {
                                self.schedule(self.render_data.breakpoint());
                            }
                        });
                    });
                let mut table = egui_extras::TableBuilder::new(ui)
                    .columns(Column::remainder(), 3)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
                table
                    .body(|mut body| {
                        self.breakpoints().drain_filter(|bp| {
                            if bp.temp() { return false };
                            let mut rem = false;
                            let (r, v) = bp.display();
                            body.row(30.0, |mut row| {
                                row.col(|ui| { if ui.button("-").clicked() { rem = true; } });
                                row.col(|ui| { ui.checkbox(&mut bp.enabled, ""); });
                                row.col(|ui| { ui.label(format!("{r:?} == {v:#06x}")); });
                            });
                            rem
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
                        .show(ui, |ui| {
                            ui.push_id("disassembly", |ui| { self.disassembly.render(&self.emu, ui); });
                        });
                    egui::Frame::group(ui.style())
                        .fill(DARK_BLACK)
                        .show(ui, |ui| {
                            let emu = self.emu.bus();
                            let bus = emu.bus();
                            egui_extras::TableBuilder::new(ui)
                                .columns(Column::exact(200.), 4)
                                //.cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .header(32., |mut header| {
                                    header.col(|ui| { ui.label("IO"); });
                                    header.col(|ui| { ui.label("Memory"); });
                                    header.col(|ui| { ui.label("Audio"); });
                                    header.col(|ui| { ui.label("Video"); });
                                })
                                .body(|mut body| {
                                    body.row(240., |mut row| {
                                        const CPU: &[IO] = &[IO::JOYP, IO::DIV, IO::TAC, IO::TIMA, IO::TMA, IO::IF, IO::IE, IO::SB, IO::SC];
                                        const MEM: &[IO] = &[IO::KEY1, IO::DMA, IO::VBK, IO::SVBK, IO::HDMA1, IO::HDMA2, IO::HDMA3, IO::HDMA4, IO::HDMA5];
                                        const AUDIO: &[IO] = &[IO::WaveRam0];
                                        const VIDEO: &[IO] = &[IO::LCDC, IO::STAT, IO::SCX, IO::SCY, IO::LY, IO::LYC, IO::DMA, IO::WX, IO::WY, IO::BGP, IO::OBP0, IO::OBP1, IO::BCPS, IO::BCPD, IO::OPRI, IO::OCPS];
                                        row.col(|ui| { io_table(ui, CPU, bus.as_ref(), "CPU"); });
                                        row.col(|ui| { io_table(ui, MEM, bus.as_ref(), "MEM");
                                            ui.push_id("BANK_TABLE", |ui| {
                                                egui_extras::TableBuilder::new(ui)
                                                    .columns(Column::exact(100.), 2)
                                                    .auto_shrink([false, false])
                                                    .body(|mut body| {
                                                        body.row(16., |mut row| {
                                                            row.col(|ui| { ui.label("ROM_H"); });
                                                            row.col(|ui| { ui.label(format!("{:#04X}", emu.mbc().rom_bank_high())); });
                                                        });
                                                        body.row(16., |mut row| {
                                                            row.col(|ui| { ui.label("ROM_L"); });
                                                            row.col(|ui| { ui.label(format!("{:#04X}", emu.mbc().rom_bank_low())); });
                                                        });
                                                        body.row(16., |mut row| {
                                                            row.col(|ui| { ui.label("RAM"); });
                                                            row.col(|ui| { ui.label(format!("{:#04X}", emu.mbc().ram_bank())); });
                                                        });
                                                    });
                                            });
                                        });
                                        row.col(|ui| { io_table(ui, AUDIO, bus.as_ref(), "AUDIO"); });
                                        row.col(|ui| { io_table(ui, VIDEO, bus.as_ref(), "VIDEO"); });
                                    });
                                });
                        });
                });
            });
    }
}
