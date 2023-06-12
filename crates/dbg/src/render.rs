use std::str::FromStr;

use egui_extras::Column;

pub use disassembly::Disassembly;
pub use memory::Viewer;
use shared::{egui::{self, Align, CentralPanel, Color32, FontFamily, Layout, Response, Ui, Widget}, Event, Events};
use shared::breakpoints::Breakpoint;
use shared::cpu::{Flags, Opcode, Reg, Value};
use shared::egui::{ScrollArea, SidePanel, Vec2};
use shared::emulator::Bus;
use shared::input::{Debug, KeyCat};
use shared::io::IO;
use shared::utils::convert::Converter;
use shared::utils::DARK_BLACK;
use shared::utils::image::ImageLoader;
use shared::winit::event::WindowEvent;

use crate::{Context, Debugger, Texture};

use super::{Emulator, Ninja};

mod disassembly;
mod memory;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Kind {
    Reg(Reg),
    Cycles,
    Instructions,
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
    op: &'static str,
    ins: Opcode,
    raw_op: String,
    raw: String,
    value: Value,
    reg: Kind,
    count: usize,
}

impl Data {
    fn breakpoint(&self) -> Breakpoint {
        match self.reg {
            Kind::Reg(reg) => Breakpoint::register(reg, self.value),
            Kind::Cycles => Breakpoint::cycles(self.count),
            Kind::Instructions => Breakpoint::instructions(self.count),
        }
    }

    fn parse_op(&mut self) {
        let opcode = u16::convert(&self.raw_op);
        let prefix = opcode & 0xFF00 == 0xCB00;
        if let Ok(opcode) = Opcode::try_from((opcode as u8, prefix)) {
            self.ins = opcode;
            self.op = shared::cpu::dbg::dbg_opcodes(opcode).1;
        }
    }

    fn parse(&mut self) {
        match (self.value, self.reg) {
            (_, Kind::Cycles | Kind::Instructions) => { self.count = usize::from_str(&self.raw).unwrap_or(0); }
            (Value::U8(_v), _) => { self.value = Value::U8(u8::convert(&self.raw)); }
            (Value::U16(_v), _) => { self.value = Value::U16(u16::convert(&self.raw)); }
        };
    }

    fn update(&mut self) {
        match (self.reg, self.value) {
            (Kind::Reg(r), Value::U16(v)) if r == Reg::A || r == Reg::B || r == Reg::C || r == Reg::D || r == Reg::E || r == Reg::F || r == Reg::H || r == Reg::L => { self.value = Value::U8(v as u8); }
            (Kind::Reg(r), Value::U8(v)) if r == Reg::AF || r == Reg::BC || r == Reg::DE || r == Reg::HL || r == Reg::PC || r == Reg::SP => { self.value = Value::U16(v as u16); }
            _ => {}
        }
    }
}

impl Default for Data {
    fn default() -> Self {
        Self { op: "NOP", ins: Opcode::Nop, raw_op: Default::default(), reg: Kind::Reg(Reg::PC), count: 0, raw: "".to_string(), value: Value::U16(0) }
    }
}


pub struct Register(&'static str, Value);

impl Widget for Register {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label(self.0);
            ui.label(match self.1 {
                Value::U8(v) => format!("{:#04x}", v),
                Value::U16(v) => format!("{:#06x}", v),
            });
        }).response
    }
}

fn io_table(ui: &mut Ui, ios: &[IO], bus: &&dyn Bus, source: &'static str, extra: impl FnOnce(&mut Ui)) {
    ScrollArea::vertical()
        .id_source("ScrollArea_".to_owned() + source)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.push_id("IOTable_".to_owned() + source, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::exact(100.), 2)
                    .auto_shrink([false, true])
                    .body(|mut body| {
                        for io in ios {
                            body.row(16., |mut row| {
                                row.col(|ui| { ui.label(format!("{:04X}:{}", *io as u16, io.name())); });
                                row.col(|ui| {
                                    let v = bus.direct_read(*io as u16);
                                    let tr = ui.label(format!("{:#04X}", v));
                                    if let Some(tooltip) = io.tooltip(v) {
                                        tr.on_hover_text(tooltip);
                                    }
                                });
                            })
                        }
                    });
            });
            extra(ui);
        });
}

impl<E: Emulator> Default for Ninja<E> {
    fn default() -> Self {
        Ninja {
            render_data: Data::default(),
            disassembly: Disassembly::new(),
            viewer: Viewer::default(),
            textures: Default::default(),
        }
    }
}

impl<E: Emulator> shared::Ui for Ninja<E> {
    type Ext = E;

    fn init(&mut self, ctx: &mut Context, _ext: &mut E) {
        self.textures.insert(Texture::Play, ctx.load_svg_bytes::<40, 40>("play", include_bytes!("../../../assets/icons/play.svg")));
        self.textures.insert(Texture::Pause, ctx.load_svg_bytes::<40, 40>("pause", include_bytes!("../../../assets/icons/pause.svg")));
        self.textures.insert(Texture::Step, ctx.load_svg_bytes::<32, 32>("step", include_bytes!("../../../assets/icons/step.svg")));
        self.textures.insert(Texture::Reset, ctx.load_svg_bytes::<40, 40>("reset", include_bytes!("../../../assets/icons/reset.svg")));
        self.textures.insert(Texture::Into, ctx.load_svg_bytes::<40, 40>("into", include_bytes!("../../../assets/icons/into.svg")));
    }

    fn draw(&mut self, ctx: &mut Context, ext: &mut E) {
        use egui::{FontFamily::Proportional, FontId, TextStyle::*};
        let mut style = (*ctx.style()).clone();
        style.visuals.override_text_color = Some(Color32::WHITE);
        style.text_styles = [
            (Heading, FontId::new(30.0, Proportional)),
            (Body, FontId::new(12.0, FontFamily::Monospace)),
            (Monospace, FontId::new(10.0, FontFamily::Monospace)),
            (Button, FontId::new(14.0, Proportional)),
            (Small, FontId::new(10.0, Proportional)),
        ].into();
        ctx.set_style(style.clone());
        SidePanel::left("left")
            .show(ctx, |ui| {
                self.viewer.render(ui, ext);
            });
        CentralPanel::default()
            .show(ctx, |ui: &mut Ui| {
                ui.horizontal_top(|ui| {
                    let sp = ui.spacing().item_spacing.x;
                    ui.spacing_mut().item_spacing.x = 3.;
                    ui.allocate_ui_with_layout(Vec2::new(500., 364.), Layout::top_down(Align::LEFT), |ui| {
                        egui::Frame::group(ui.style()).fill(DARK_BLACK)
                            .show(ui, |ui| {
                                ui.columns(6, |uis| {
                                    let flags = ext.cpu_register(Reg::F);
                                    uis[0].vertical(|ui| {
                                        ui.add(Register("A", ext.cpu_register(Reg::A)));
                                        ui.add(Register("F", flags));
                                    });
                                    uis[1].vertical(|ui| {
                                        ui.add(Register("B", ext.cpu_register(Reg::B)));
                                        ui.add(Register("C", ext.cpu_register(Reg::C)));
                                    });
                                    uis[2].vertical(|ui| {
                                        ui.add(Register("D", ext.cpu_register(Reg::D)));
                                        ui.add(Register("E", ext.cpu_register(Reg::E)));
                                    });
                                    uis[3].vertical(|ui| {
                                        ui.add(Register("H", ext.cpu_register(Reg::H)));
                                        ui.add(Register("L", ext.cpu_register(Reg::L)));
                                    });
                                    uis[4].vertical(|ui| {
                                        ui.add(Register("SP", ext.cpu_register(Reg::SP)));
                                        ui.add(Register("PC", ext.cpu_register(Reg::PC)));
                                    });
                                    uis[5].with_layout(Layout::top_down(Align::Center), |ui| {
                                        //ui.label("Flags");
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
                        egui::Frame::group(ui.style())
                            .fill(DARK_BLACK)
                            .show(ui, |ui| {
                                ui.push_id("disassembly", |ui| { self.disassembly.render(ext, ui); });
                            });
                    });
                    ui.spacing_mut().item_spacing.x = sp;
                    ui.allocate_ui_with_layout(Vec2::new(340., 366.), Layout::top_down(Align::LEFT), |ui| {
                        egui::Frame::group(ui.style())
                            .fill(DARK_BLACK)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 16.;
                                    let sz: Vec2 = (32., 32.).into();
                                    let pause = egui::ImageButton::new(self.tex(Texture::Pause), (40., 40.)).frame(false);
                                    let reset = egui::ImageButton::new(self.tex(Texture::Reset), (40., 40.)).frame(false);
                                    let play = egui::ImageButton::new(self.tex(Texture::Play), (40., 40.)).frame(false);
                                    let step = egui::ImageButton::new(self.tex(Texture::Step), sz).frame(false);
                                    let into = egui::ImageButton::new(self.tex(Texture::Into), sz).frame(false);
                                    ui.allocate_ui_with_layout(Vec2::splat(64.), Layout::top_down(Align::Center), |ui| {
                                        let sp = ext.speed();
                                        if ui.add(egui::Button::new("+")).clicked() { ext.speedup(); }
                                        let text = match sp {
                                            0 => egui::Label::new("Normal"),
                                            n @ 1.. => egui::Label::new(format!("{}x", 1. + n as f32 / 5.)),
                                            n => egui::Label::new(format!("1/{}", (1 << -n)))
                                        };
                                        ui.add(text);
                                        if ui.add(egui::Button::new("-")).clicked() { ext.speeddown(); }
                                    });
                                    if ui.add(into).clicked() { ext.step_into(&mut self.disassembly); };
                                    if ui.add(pause).clicked() { ext.pause(); };
                                    if ui.add(play).clicked() { Debugger::<E>::play(ext, &mut self.disassembly); };
                                    if ui.add(step).clicked() { ext.step(&mut self.disassembly) };
                                    if ui.add(reset).clicked() { ext.reset(); };
                                });
                                ui.horizontal(|ui| {
                                    if ui.add(egui::TextEdit::singleline(&mut self.render_data.raw_op).desired_width(64.)).changed() {
                                        self.render_data.parse_op();
                                    }
                                    if ui.button("BREAK").clicked() {
                                        ext.schedule(Breakpoint::instruction(self.render_data.ins));
                                    }
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
                                        ext.schedule(self.render_data.breakpoint());
                                    }
                                    let mut and = ext.breakpoints().and();
                                    ui.toggle_value(&mut and, "And");
                                    ext.breakpoints().set_and(and);
                                });
                                egui_extras::TableBuilder::new(ui)
                                    .columns(Column::remainder(), 3)
                                    .striped(true)
                                    .vscroll(true)
                                    .auto_shrink([false; 2])
                                    .cell_layout(Layout::left_to_right(Align::Center))
                                    .body(|mut body| {
                                        ext.breakpoints()
                                            .bp_mut()
                                            .drain_filter(|bp| {
                                                if bp.temp() { return false; };
                                                let mut rem = false;
                                                body.row(30.0, |mut row| {
                                                    row.col(|ui| { if ui.button("-").clicked() { rem = true; } });
                                                    row.col(|ui| { ui.checkbox(&mut bp.enabled, ""); });
                                                    row.col(|ui| { ui.label(bp.display()); });
                                                });
                                                rem
                                            });
                                    });
                            });
                    });
                });
                egui::Frame::group(ui.style()) // IOregs
                    .fill(DARK_BLACK)
                    .show(ui, |ui| {
                        let boxed_bus = ext.bus();
                        let bus = boxed_bus.as_ref();
                        egui_extras::TableBuilder::new(ui)
                            .columns(Column::exact(202.), 4)
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
                                    const AUDIO: &[IO] = &[IO::PCM12, IO::PCM34,
                                        IO::NR50, IO::NR51, IO::NR52, IO::NR10, IO::NR11, IO::NR12, IO::NR13, IO::NR14, IO::NR21, IO::NR22, IO::NR23, IO::NR24,
                                        IO::NR30, IO::NR31, IO::NR32, IO::NR33, IO::NR34, IO::NR41, IO::NR42, IO::NR43, IO::NR44,
                                        IO::WaveRam0, IO::WaveRam1, IO::WaveRam2, IO::WaveRam3, IO::WaveRam4, IO::WaveRam5, IO::WaveRam6, IO::WaveRam7,
                                        IO::WaveRam8, IO::WaveRam9, IO::WaveRamA, IO::WaveRamB, IO::WaveRamC, IO::WaveRamD, IO::WaveRamE, IO::WaveRamF];
                                    const VIDEO: &[IO] = &[IO::LCDC, IO::STAT, IO::SCX, IO::SCY, IO::LY, IO::LYC, IO::DMA, IO::WX, IO::WY, IO::BGP, IO::OBP0, IO::OBP1, IO::BCPS, IO::BCPD, IO::OPRI, IO::OCPS];
                                    row.col(|ui| { io_table(ui, CPU, bus, "CPU", |_| {}); });
                                    row.col(|ui| {
                                        io_table(ui, MEM, bus, "MEM", |ui: &mut Ui| {
                                            ui.push_id("BANK_TABLE", |ui| {
                                                egui_extras::TableBuilder::new(ui)
                                                    .columns(Column::exact(100.), 2)
                                                    .auto_shrink([false, false])
                                                    .body(|mut body| {
                                                        body.row(16., |mut row| {
                                                            row.col(|ui| { ui.label("ROM"); });
                                                            row.col(|ui| { ui.label(format!("{:#04X}", ext.mbc().rom_bank())); });
                                                        });
                                                        body.row(16., |mut row| {
                                                            row.col(|ui| { ui.label("RAM"); });
                                                            row.col(|ui| { ui.label(format!("{:#04X}", ext.mbc().ram_bank())); });
                                                        });
                                                    });
                                            });
                                        })
                                    });
                                    row.col(|ui| { io_table(ui, AUDIO, bus, "AUDIO", |_| {}); });
                                    row.col(|ui| { io_table(ui, VIDEO, bus, "VIDEO", |_| {}); });
                                });
                            });
                    });
            });
    }

    fn handle(&mut self, event: &Event, _ctx: &mut Context, ext: &mut E) {
        match event {
            Event::UserEvent(Events::Loaded) => self.disassembly.reload(),
            Event::WindowEvent { event: WindowEvent::MouseWheel { .. }, .. } => self.disassembly.fixed(&ext),
            Event::UserEvent(Events::Press(KeyCat::Dbg(key))) => {
                match key {
                    Debug::Pause => ext.pause(),
                    Debug::Reset => ext.reset(),
                    Debug::Step => ext.step(&mut self.disassembly),
                    Debug::Run => Debugger::play(ext, &mut self.disassembly)
                }
            }
            _ => {}
        }
    }
}

