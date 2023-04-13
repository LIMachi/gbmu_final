use std::ops::Range;
use shared::breakpoints::{Breakpoint, Breakpoints, Value};
use shared::emulator::Emulator;

use shared::egui;
use shared::egui::{Color32, Label, Response, ScrollArea, Sense, TextStyle, Ui, Vec2, Widget};
use shared::egui::RichText;
use shared::mem::*;
use crate::render::DARK_BLACK;

struct View {
    mem: &'static str,
    range: Range<u16>
}

impl View {
    pub fn new(mem: &'static str, range: Range<u16>) -> Self {
        Self { mem, range }
    }
}

#[derive(Clone)]
struct ViewerOptions {
    zero_color: Color32,
    address_color: Color32,
    highlight_color: Color32,
    text_style: TextStyle,
    address_text_style: TextStyle
}

impl Default for ViewerOptions {
    fn default() -> Self {
        Self {
            zero_color: Color32::from_gray(80),
            address_color: Color32::from_rgb(125, 0, 125),
            highlight_color: Color32::from_rgb(0, 140, 140),
            text_style: TextStyle::Monospace,
            address_text_style: TextStyle::Monospace
        }
    }
}

pub struct Viewer {
    options: ViewerOptions,
    ranges: [View; 7],
    current: usize,
    hover: Option<u16>,
    input: String,
    value: Value
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            options: Default::default(),
            ranges: [
                View::new("ROM", ROM..VRAM),
                View::new("VRAM", VRAM..SRAM),
                View::new("SRAM", SRAM..RAM),
                View::new("RAM", RAM..ECHO),
                View::new("OAM", OAM..UN_1),
                View::new("IO", IO..HRAM),
                View::new("HRAM", HRAM..END)
            ],
            current: 0,
            hover: None,
            input: String::new(),
            value: Value::Any
        }
    }
}

const COLUMNS: u16 = 16;

struct ContextMenu<'a> {
    value: &'a mut Value,
    input: &'a mut String,
    breaks: &'a mut Breakpoints,
    addr: u16
}

impl<'a> Widget for ContextMenu<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.label("Break on ");
            ui.radio_value(self.value, Value::Any, Value::Any.sym());
            ui.radio_value(self.value, Value::Eq(0), Value::Eq(0).sym());
            ui.radio_value(self.value, Value::Not(0), Value::Not(0).sym());
            ui.radio_value(self.value, Value::And(0), Value::And(0).sym());
        });
        if self.value.param() {
            ui.horizontal(|ui| {
                ui.label("Value: ");
                if ui.text_edit_singleline(self.input).changed() {
                    self.value.parse(self.input);
                }
            });
        }
        use shared::breakpoints::Access;
        let rr = ui.button("Read");
        let rw = ui.button("Write");
        let rrw = ui.button("R/W");
        let access = if rr.clicked() { Access::read(self.addr, *self.value) }
        else if rw.clicked() { Access::write(self.addr, *self.value) }
        else if rrw.clicked() { Access::rw(self.addr, *self.value) }
        else { return rr | rw | rrw };
        ui.close_menu();
        self.breaks.schedule(Breakpoint::access(access));
        rr | rw | rrw
    }
}

impl Viewer {
    fn current(&self) -> &'static str {
        self.ranges[self.current].mem
    }

    fn get_line_height(&self, ui: &egui::Ui) -> f32 {
        ui.text_style_height(&self.options.address_text_style)
            .max(ui.text_style_height(&self.options.text_style))
    }

    pub fn render(&mut self, ui: &mut egui::Ui, emu: &mut dyn Emulator) {
        egui::Frame::group(ui.style())
            .fill(DARK_BLACK)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("View: ");
                    ui.menu_button(self.current(), |ui| {
                       for i in 0..self.ranges.len() {
                           if ui.button(self.ranges[i].mem).clicked() {
                               self.current = i;
                           }
                       }
                    });
                });
                let ViewerOptions {
                    zero_color,
                    address_color,
                    highlight_color,
                    text_style,
                    address_text_style,
                } = self.options.clone();
                let space = &self.ranges[self.current];
                ui.separator();
                let scroll = ScrollArea::vertical()
                    .id_source(self.current())
                    .max_height(f32::INFINITY)
                    .auto_shrink([false, true]);
                let height = self.get_line_height(ui);
                let max_lines = (space.range.len() + 15) / 16;
                scroll.show_rows(ui, height, max_lines, |ui, range| {
                    egui::Grid::new("viewer_grid")
                        .striped(true)
                        .spacing(Vec2::new(15., ui.style().spacing.item_spacing.y))
                        .show(ui, |ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.style_mut().spacing.item_spacing.x = 3.0;
                            let mut hover = None;
                            for row in range {
                                let addr = space.range.start + COLUMNS * row as u16;
                                let mut print = addr;
                                let mut color = address_color;
                                if let Some(h) = self.hover {
                                    if h & 0xFFF0 == addr { print = h; color = Color32::GREEN }
                                }
                                let text = RichText::new(format!("0x{:04X}:", print)).color(color)
                                    .text_style(address_text_style.clone());
                                ui.label(text);
                                for col in 0..2 {
                                    let st =  addr + 8 * col;
                                    ui.horizontal(|ui| {
                                        for c in 0..8 {
                                            let addr = st + c;
                                            if !space.range.contains(&addr) { break; }
                                            let v = emu.bus().read(addr);
                                            let text = format!("{:02X}", v);
                                            let text = RichText::new(text).text_style(text_style.clone())
                                                .color(if Some(addr) == self.hover { highlight_color } else if v == 0 { zero_color } else { ui.style().visuals.text_color() });
                                            let label = Label::new(text).sense(Sense::click());
                                            let ret = ui.add(label);
                                            if ret.hovered() { hover = Some(addr) }
                                            ret.context_menu(|ui| {
                                                ui.add(ContextMenu {
                                                    value: &mut self.value,
                                                    input: &mut self.input,
                                                    breaks: &mut emu.breakpoints(),
                                                    addr
                                                });
                                            });
                                        }
                                    });
                                }
                                ui.end_row();
                            }
                            self.hover = hover;
                        });
                });
            });
    }
}
