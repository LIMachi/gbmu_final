use std::collections::HashMap;

use egui::Ui;

use shared::{egui, Events, Handle};
use shared::egui::{Align, Context, Direction, Image, Layout, Margin, Rect, Response, Rounding, Sense, Separator, TextureHandle, Vec2, Widget};
use shared::rom::Rom;
use shared::utils::DARK_BLACK;
use shared::utils::image::ImageLoader;

use crate::app::{Menu, Texture};
use crate::app::watcher::Event;
use crate::emulator::Emulator;

const ROM_GRID: f32 = 128.;

struct RomView<'a> {
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
    fn new(rom: &'a Rom, textures: &HashMap<Texture, TextureHandle>) -> Self {
        let handle = rom.cover.as_ref().and_then(|x| textures.get(&Texture::Cover(x.clone()))).map(|x| x.clone());
        Self { rom, handle }
    }
}

impl shared::Ui for Menu {
    type Ext = Emulator;

    fn init(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        self.textures.insert(Texture::Add, ctx.load_svg::<40, 40>("add", "assets/icons/add.svg"));
        self.textures.insert(Texture::Debug, ctx.load_svg::<40, 40>("debug", "assets/icons/debug.svg"));
        self.textures.insert(Texture::Settings, ctx.load_svg::<40, 40>("settings", "assets/icons/settings.svg"));
        self.textures.insert(Texture::Spritesheet, ctx.load_svg::<40, 40>("spritesheet", "assets/icons/palette.svg"));
        self.textures.insert(Texture::Save, ctx.load_svg::<40, 40>("save", "assets/icons/save.svg"));
        self.textures.insert(Texture::Nosave, ctx.load_svg::<40, 40>("nosave", "assets/icons/stop.svg"));
        for path in &emu.roms.paths {
            self.watcher.add_path(path.clone());
            self.search(path);
        }
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        for evt in self.watcher.iter() {
            match evt {
                Event::Reload(path) => {
                    self.roms.drain_filter(|x, _| x.starts_with(&path));
                    self.search(&path)
                }
            }
        }
        while let Ok((path, mut rom)) = self.receiver.try_recv() {
            self.add_rom(path, rom, ctx);
        }
        let style = ctx.style();
        ctx.set_debug_on_hover(true);
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
                        let w = ui.available_width();
                        egui::Grid::new("roms").show(ui, |ui| {
                            let mut n = 1;
                            for rom in self.roms.values() {
                                if n as f32 * (ROM_GRID + ui.spacing().item_spacing.x * 2.) + ui.spacing().scroll_bar_width + ui.spacing().scroll_bar_outer_margin > w {
                                    ui.end_row();
                                    n = 1;
                                }
                                if ui.add(RomView::new(rom, &self.textures)).clicked() {
                                    emu.proxy.send_event(Events::Play(rom.clone())).ok();
                                }
                                n += 1;
                            }
                        });
                    });
            });
    }
}
