use std::path::{Path, PathBuf};

use shared::{egui, Events, Handle};
use shared::egui::{Align, Context, Layout, Margin, Rounding, Separator};
use shared::utils::image::ImageLoader;
pub(crate) use shelves::{Shelf, ShelfItem};

use crate::app::{AppConfig, Event, Menu, Texture};
use crate::emulator::Emulator;

mod rom;
mod shelves;
pub mod state;

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
            self.roms.new_root(PathBuf::from(path), None);
        }
        self.states.new_root(AppConfig::state_path(), Some(String::from("Save States")));
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        for evt in self.roms.update() {
            match evt {
                Event::Delete(path) => {
                    emu.roms.paths.retain(|x| Path::new(x) != &path);
                }
                Event::Added(root, path, mut rom) => {
                    self.add_cover(&path, &mut rom, ctx);
                    self.roms.add_item(root, path, rom);
                }
                _ => {}
            }
        }
        for evt in self.states.update() {
            if let Event::Added(root, path, mut state) = evt {
                self.add_cover(&path, &mut state, ctx);
                self.states.add_item(root, path, state);
            }
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
                        if emu.console.active() {
                            if ui.add(save_state).clicked() { emu.save_state(); }
                        }
                        if emu.is_running() {
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
                        for shelf in &mut self.states.shelves {
                            ui.add(shelf.view(emu, &self.textures, self.states.watcher.tx())
                                .can_remove_root(false));
                        }
                        ui.separator();
                        for shelf in &mut self.roms.shelves {
                            ui.add(shelf.view(emu, &self.textures, self.roms.watcher.tx()));
                        }
                    });
            });
    }
}
