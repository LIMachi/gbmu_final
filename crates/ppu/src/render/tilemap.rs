use std::collections::HashMap;
use shared::egui::{ColorImage, Image, Response, TextureHandle, Ui, Widget};
use crate::render::Textures;

pub struct Tilemap<'a>(pub &'a HashMap<Textures, TextureHandle>);

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
                        let tex = self.0.get(&Textures::Tile(addr)).unwrap().id();
                        if i == 15 { ui.spacing_mut().item_spacing.x = 8. }
                        ui.add(Image::new(tex, [32., 32.]));
                    }
                    let blank = self.0.get(&Textures::Blank).unwrap().id();
                    ui.spacing_mut().item_spacing.x = 1.;
                    for i in 0..16 {
                        let addr = i + j * 16 + 384;
                        let tex = self.0.get(&Textures::Tile(addr)).map(|x| x.id()).unwrap_or(blank);
                        ui.add(Image::new(tex, [32., 32.]));
                    }
                });
            }
        }).response
    }
}
