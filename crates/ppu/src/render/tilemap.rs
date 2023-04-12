use std::collections::HashMap;
use shared::egui::{Image, Response, TextureHandle, Ui, Widget};
use crate::render::Textures;

pub struct Tilemap<'a>(pub(crate) &'a mut crate::UiData);

impl Widget for Tilemap<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.spacing_mut().item_spacing.x = 1.;
        ui.spacing_mut().item_spacing.y = 1.;
        ui.vertical(|ui| {
            for j in 0..24 {
                ui.spacing_mut().item_spacing.y = 1.;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.;
                    for i in 0..16 {
                        let addr = i + j * 16;
                        let tex = self.0.get(addr).id();
                        if i == 15 { ui.spacing_mut().item_spacing.x = 8. }
                        ui.add(Image::new(tex, [32., 32.]));
                    }
                    let blank = self.0.tex(Textures::Blank).unwrap().id();
                    ui.spacing_mut().item_spacing.x = 1.;
                    for i in 0..16 {
                        let addr = i + j * 16 + 384;
                        let tex = self.0.get(addr).id();
                        ui.add(Image::new(tex, [32., 32.]));
                    }
                });
            }
        }).response
    }
}
