use std::fmt::format;
use std::ops::Range;
use std::thread::current;
use shared::egui;
use shared::egui::{Color32, Label, ScrollArea, Sense, TextStyle, Vec2};
use shared::egui::RichText;
use shared::mem::*;
use crate::Bus;
use super::render::DARK_BLACK;

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
    ranges: [View; 6],
    current: usize,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            options: Default::default(),
            ranges: [
                View::new("ROM", ROM..VRAM),
                View::new("VRAM", VRAM..SRAM),
                View::new("EXT_RAM", SRAM..RAM),
                View::new("RAM", RAM..ECHO),
                View::new("OAM", OAM..UN_1),
                View::new("HRAM", HRAM..END)
            ],
            current: 0
        }
    }
}

const COLUMNS: u16 = 16;

impl Viewer {
    fn current(&self) -> &'static str {
        self.ranges[self.current].mem
    }

    fn get_line_height(&self, ui: &egui::Ui) -> f32 {
        ui.text_style_height(&self.options.address_text_style)
            .max(ui.text_style_height(&self.options.text_style))
    }

    pub fn render(&mut self, ui: &mut egui::Ui, bus: &Box<&dyn Bus>) {
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
                    ..
                } = self.options.clone();
                let space = &self.ranges[self.current];
                ui.separator();
                let mut scroll = ScrollArea::vertical()
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
                            for row in range {
                                let addr = space.range.start + COLUMNS * row as u16;
                                let text = RichText::new(format!("0x{:04X}:", addr))
                                    .text_style(address_text_style.clone());
                                ui.label(text);
                                for col in 0..2 {
                                    let st =  addr + 8 * col;
                                    ui.horizontal(|ui| {
                                        for c in 0..8 {
                                            let addr = st + c;
                                            if !space.range.contains(&addr) { break; }
                                            let v = bus.read(addr);
                                            let text = format!("{:02X}", v);
                                            let mut text = RichText::new(text).text_style(text_style.clone())
                                                .color(if v == 0 { zero_color } else { ui.style().visuals.text_color() });
                                            let res = ui.add(Label::new(text).sense(Sense::click()));
                                        }
                                    });
                                }
                                ui.end_row();
                            }
                        });
                });
            });
    }
}
