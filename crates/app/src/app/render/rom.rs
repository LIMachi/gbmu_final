use std::collections::HashMap;

use shared::egui;
use shared::egui::{Direction, Image, Layout, Rect, Response, Sense, TextureHandle, Ui, Vec2, Widget};
use shared::rom::Rom;
use shared::utils::DARK_BLACK;

use crate::app::Texture;

pub const ROM_GRID: f32 = 128.;

pub struct RomView<'a> {
    rom: &'a Rom,
    handle: Option<TextureHandle>,
}

impl<'a> Widget for RomView<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(egui::Vec2::new(ROM_GRID, ROM_GRID + 16.), Sense::click());
        let img = Rect::from_min_size(response.rect.min, Vec2::splat(ROM_GRID));
        let mut ui = if let Some(x) = self.handle {
            Image::new(x.id(), (ROM_GRID, ROM_GRID)).paint_at(ui, img);
            let mut pos = img.min;
            pos.y += ROM_GRID;
            ui.child_ui(Rect::from_min_size(pos, Vec2::new(ROM_GRID, 16.)), Layout::centered_and_justified(Direction::LeftToRight))
        } else {
            ui.child_ui(response.rect, Layout::centered_and_justified(Direction::LeftToRight))
        };
        egui::Frame::none()
            .fill(DARK_BLACK)
            .show(&mut ui, |ui| {
                let title = &self.rom.header.title;
                ui.label(title);
            });
        response
    }
}

impl<'a> RomView<'a> {
    pub(crate) fn new(rom: &'a Rom, textures: &HashMap<Texture, TextureHandle>) -> Self {
        let handle = rom.cover.as_ref().and_then(|x| textures.get(&Texture::Cover(x.clone()))).map(|x| x.clone());
        Self { rom, handle }
    }
}
