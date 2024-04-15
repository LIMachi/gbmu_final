use shared::egui::{Image, Response, TextureId, Ui, Widget};
use shared::emulator::Emulator;

use crate::render::Textures;
use crate::VramViewer;

pub struct Tilemap<'a, E: Emulator>(pub(crate) &'a mut VramViewer<E>);

struct Tile(Image, u8, bool);

impl Widget for Tile {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.add(self.0)
            .on_hover_text(format!("tile: {:02X}{}", self.1, if self.2 { "(H)" } else { "" }))
    }
}

impl Tile {
    pub fn new(tex: TextureId, n: usize) -> Self {
        let img = Image::new(tex, [32., 32.]);
        Tile(img, n as u8, n > 0xFF)
    }
}

impl<E: Emulator> Widget for Tilemap<'_, E> {
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
                        if i == 15 { ui.spacing_mut().item_spacing.x = 8. }
                        ui.add(Tile::new(self.0.draw_tex(addr).unwrap(), addr));
                    }
                    let blank = self.0.tex(Textures::Blank).unwrap().id();
                    ui.spacing_mut().item_spacing.x = 1.;
                    for i in 0..16 {
                        let n = i + j * 16;
                        ui.add(Tile::new(self.0.draw_tex(n + 384).unwrap_or(blank), n));
                    }
                });
            }
        }).response
    }
}
