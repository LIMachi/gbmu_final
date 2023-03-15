use std::collections::HashMap;
use shared::egui;
use shared::egui::{Color32, Image, Response, Stroke, TextureHandle, Ui, Vec2, Widget};
use crate::render::Textures;

pub struct BgMap<'a>(pub &'a HashMap<Textures, TextureHandle>, pub(crate) &'a super::Ppu, pub(crate) &'a egui::Context);

impl Widget for BgMap<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.spacing_mut().item_spacing.x = 1.;
        ui.spacing_mut().item_spacing.y = 1.;
        egui::Area::new("scrolled_area")
            .fixed_pos([self.1.sc.x as f32 + ui.available_rect_before_wrap().min.x, self.1.sc.y as f32 + ui.available_rect_before_wrap().min.y])
            .movable(false)
            .show(self.2, |ui| {
                egui::Frame::none()
                    .stroke(Stroke::new(2., Color32::BLACK))
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                       ui.allocate_space(Vec2::new(500., 450.));
                    });
            });
        ui.vertical(|ui| {
            for j in 0..32 {
                ui.spacing_mut().item_spacing.y = 1.;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.;
                    for i in 0..32 {
                        let addr = i + j * 32 + if self.1.lcdc.bg_area() { 0x1C00 } else { 0x1800 };
                        let tile = self.1.vram.inner().read_bank(addr, 0);
                        let tile = if self.1.lcdc.relative_addr() {
                            (256 + (tile as i8) as isize) as usize
                        } else {
                            tile as usize
                        };
                        let tex = self.0.get(&Textures::Tile(tile as usize)).unwrap().id();
                        ui.add(Image::new(tex, [24., 24.]));
                    }
                });
            }
        }).response
    }
}
