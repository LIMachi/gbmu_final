use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;

use shared::egui;
use shared::egui::{Context, Direction, Image, Layout, Rect, Response, Sense, TextureHandle, TextureOptions, Ui, Vec2, Widget};
use shared::utils::DARK_BLACK;

use crate::app::render::rom::ROM_GRID;
use crate::app::render::shelves::ShelfView;
use crate::app::Texture;
pub use crate::emulator::State;

pub struct StateView<'a> {
    state: &'a State,
    handle: TextureHandle,
}

impl<'a> Widget for StateView<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_response(egui::Vec2::new(ROM_GRID, ROM_GRID + 16.), Sense::click());
        let img = Rect::from_min_size(response.rect.min, Vec2::splat(ROM_GRID));
        Image::new(self.handle.id(), (ROM_GRID, ROM_GRID)).paint_at(ui, img);
        let mut pos = img.min;
        pos.y += ROM_GRID;
        let mut ui = ui.child_ui(Rect::from_min_size(pos, Vec2::new(ROM_GRID, 16.)), Layout::centered_and_justified(Direction::LeftToRight));
        egui::Frame::none()
            .fill(DARK_BLACK)
            .show(&mut ui, |ui| {
                // let title = &self.rom.header.title;
                // ui.label(title);
            });
        response
    }
}


impl super::ShelfItem for State {
    fn render(&self, textures: &HashMap<Texture, TextureHandle>, ui: &mut Ui) -> Response {
        ui.add(StateView {
            state: self,
            handle: textures.get(&Texture::Cover(self.cover.clone().unwrap())).cloned().unwrap(),
        })
    }

    fn clicked(&self, shelf: &mut ShelfView<Self>) where Self: Sized {
        shelf.emu.load_state(Some(self));
    }

    fn extensions() -> &'static [&'static str] { &["state"] }

    fn load_from_path(path: &Path) -> std::io::Result<Self> where Self: Sized {
        std::fs::File::open(path)
            .and_then(|mut x| {
                use std::io::Read;
                let mut v = Vec::new();
                x.read_to_end(&mut v)?;
                bincode::deserialize(&v).map_err(|e| {
                    std::io::Error::new(ErrorKind::InvalidData, format!("deserialize failed {e:?}"))
                })
            })
    }

    fn set_cover(&mut self, cover: String) {
        self.cover = Some(cover);
    }

    fn load_cover(&self, ctx: &Context) -> Option<TextureHandle> {
        Some(ctx.load_texture(&self.path, self.preview.image(), TextureOptions::LINEAR))
    }
}
