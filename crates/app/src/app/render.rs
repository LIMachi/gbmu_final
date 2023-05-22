use std::path::{Path, PathBuf};

use shared::{egui, Events, Handle};
use shared::egui::{Align, Context, Layout, Margin, Rounding, Separator};
use shared::utils::image::ImageLoader;
pub use shelves::Shelf;

use crate::app::{Menu, Texture};
use crate::emulator::Emulator;

use super::Event;

mod rom;
mod shelves;

impl shared::Ui for Menu {
    type Ext = Emulator;

    fn init(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        self.textures.insert(Texture::Add, ctx.load_svg_bytes::<40, 40>("add", include_bytes!("../../../../assets/icons/add.svg")));
        self.textures.insert(Texture::Debug, ctx.load_svg_bytes::<40, 40>("debug", include_bytes!("../../../../assets/icons/debug.svg")));
        self.textures.insert(Texture::Settings, ctx.load_svg_bytes::<40, 40>("settings", include_bytes!("../../../../assets/icons/settings.svg")));
        self.textures.insert(Texture::Spritesheet, ctx.load_svg_bytes::<40, 40>("spritesheet", include_bytes!("../../../../assets/icons/palette.svg")));
        self.textures.insert(Texture::Save, ctx.load_svg_bytes::<40, 40>("save", include_bytes!("../../../../assets/icons/save.svg")));
        self.textures.insert(Texture::Nosave, ctx.load_svg_bytes::<40, 40>("nosave", include_bytes!("../../../../assets/icons/stop.svg")));
        self.textures.insert(Texture::SaveState, ctx.load_svg_bytes::<40, 40>("save_state", include_bytes!("../../../../assets/icons/s.svg")));
        for path in &emu.roms.paths {
            self.shelves.push(Shelf::root(PathBuf::from(path)));
            self.watcher.add_path(path.clone());
            self.search(path);
        }
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        let mut rem = vec![];
        for evt in self.watcher.iter() {
            match evt {
                Event::Delete(path) => {
                    emu.roms.paths.drain_filter(|x| Path::new(x) == &path);
                    self.shelves.drain_filter(|x| x.has_root(&path));
                    rem.push(path);
                }
                Event::Reload(path) => {
                    if let Some(shelf) = self.shelves.iter_mut()
                        .find(|x| x.has_root(&path)) {
                        shelf.clear();
                    }
                    self.search(&path)
                }
            }
        }
        for path in rem {
            self.watcher.remove_path(&path);
        }
        while let Ok((root, path, rom)) = self.receiver.try_recv() {
            self.add_rom(root, path, rom, ctx);
        }
        let style = ctx.style();
        let frame = egui::Frame::side_top_panel(&style)
            .inner_margin(Margin::symmetric(8., 8.))
            .rounding(Rounding::none());
        egui::containers::TopBottomPanel::top("menu")
            .frame(frame)
            .show(ctx, |ui| {
                ui.columns(2, |uis| {
                    let [l, r] = match uis {
                        [l, r] => [l, r],
                        _ => unreachable!()
                    };
                    let debug = egui::ImageButton::new(self.tex(Texture::Debug), (28., 28.)).frame(false);
                    let spritesheet = egui::ImageButton::new(self.tex(Texture::Spritesheet), (28., 28.)).frame(false);
                    let setting = egui::ImageButton::new(self.tex(Texture::Settings), (24., 24.)).frame(false);
                    let add = egui::ImageButton::new(self.tex(Texture::Add), (32., 32.)).frame(false);
                    let save = egui::ImageButton::new(self.tex(Texture::Save), (32., 32.)).frame(false);
                    let nosave = egui::ImageButton::new(self.tex(Texture::Nosave), (32., 32.)).frame(false);
                    let save_state = egui::ImageButton::new(self.tex(Texture::SaveState), (32., 32.)).frame(false);
                    l.with_layout(Layout::default(), |ui| {
                        if ui.add(add).clicked() {
                            if let Some(file) = rfd::FileDialog::new().set_directory("/").pick_folder() {
                                self.add_path(&mut emu.roms, file);
                            }
                        }
                    });
                    r.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(debug).clicked() { emu.proxy.send_event(Events::Open(Handle::Debug)).ok(); };
                        if ui.add(spritesheet).clicked() { emu.proxy.send_event(Events::Open(Handle::Sprites)).ok(); };
                        if ui.add(setting).clicked() { emu.proxy.send_event(Events::Open(Handle::Settings)).ok(); };
                        ui.add(Separator::default().vertical().spacing(4.));
                        if emu.is_running() {
                            if ui.add(save_state).clicked() { emu.console.save_state(); }
                            ui.add(Separator::default().vertical().spacing(4.));
                            if ui.add(save).clicked() { emu.console.bus.save(false); }
                            if ui.add(nosave).clicked() { emu.stop(false); }
                        }
                    });
                })
            });
        egui::containers::CentralPanel::default()
            .show(ctx, |ui| {
                egui::containers::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for shelf in &mut self.shelves {
                            ui.add(shelf.view(emu, &self.textures, self.watcher.tx()));
                        }
                    });
            });
    }
}
