
use shared::egui;
use super::*;

struct Sprite<'a>(&'a mem::oam::Sprite, TextureId, TextureId);

impl Widget for Sprite<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::none()
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let sp = ui.spacing_mut().item_spacing.y;
                    ui.spacing_mut().item_spacing.y = 0.;
                    ui.add(egui::Image::new(self.1, [64., 64.]));
                    ui.add(egui::Image::new(self.2, [64., 64.]));
                    ui.spacing_mut().item_spacing.y = sp;
                });
                ui.vertical(|ui| {
                    let sp = ui.spacing_mut().item_spacing.y;
                    ui.spacing_mut().item_spacing.y = 8.;
                    ui.label(format!("{:03}", self.0.y));
                    ui.label(format!("{:03}", self.0.x));
                    ui.label(format!("{:03}", self.0.tile));
                    ui.label(format!("{:#04X}", self.0.flags));
                    ui.spacing_mut().item_spacing.y = sp;
                })
            }).response
    }
}

pub struct Oam<'a, E: Emulator + MemAccess>(pub &'a HashMap<Textures, TextureHandle>, pub(crate) &'a mut E);

impl<E: Emulator + MemAccess> Widget for Oam<'_, E> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            for j in 0..5 {
                let sp = ui.spacing().item_spacing.y;
                ui.spacing_mut().item_spacing.y = 32.;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.y = sp;
                    for i in 0..8 {
                        let index = i + j * 8;
                        let sprite = self.1.sprite(index);
                        let h = self.0.get(&if sprite.x > 0 && sprite.y > 0 { Textures::Tile(sprite.tile as usize) } else { Textures::Placeholder }).unwrap().id();
                        let h2 = self.0.get(&if self.1.lcdc.obj_tall() { Textures::Tile(sprite.tile as usize + 1) } else { Textures::Placeholder }).unwrap().id();
                        if ui.add(Sprite(&sprite, h, h2)).hovered() {
                        }
                    }
                });
            }
        }).response
    }
}
