use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use shared::egui::{self, Align, Color32, Context, Direction, Image, Layout, Margin, Rect, Response, Rounding, Sense, TextureHandle, TextureId, Ui, Vec2, Widget};
use shared::rom::Rom;
use shared::utils::image::ImageLoader;

use shared::serde::{Serialize, Deserialize};
use shared::{Events, Handle};
use shared::audio_settings::AudioSettings;
use shared::breakpoints::Breakpoint;
use shared::input::Keybindings;
use crate::emulator::Emulator;

const DARK_BLACK: Color32 = Color32::from_rgb(0x23, 0x27, 0x2A);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Texture {
    Settings,
    Spritesheet,
    Debug,
    Add,
    Cover(String)
}

// TODO serde Deserialize/Serialize + serde_toml
// paths are OS specific paths to rom directories/files mixup
// each of these paths should be handed to Rom::find_roms for recursive dir traversal and rom retrieval
// if no roms.conf file is found (use default directories later, for now project root path)
#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename = "roms")]
pub struct RomConfig {
    paths: Vec<String>
}

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(rename = "debug")]
pub struct DbgConfig {
    pub breaks: Vec<Breakpoint>
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub roms: RomConfig,
    #[serde(default)]
    pub debug: DbgConfig,
    #[serde(default)]
    pub keys: Keybindings,
    #[serde(default)]
    pub sound_device: apu::SoundConfig,
    #[serde(default)]
    pub audio_settings: AudioSettings,
    #[serde(default)]
    pub mode: super::settings::Mode,
    #[serde(default)]
    pub emu: super::emulator::EmuSettings,
    #[serde(default)]
    pub bios: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        serde_any::from_file("gbmu.ron").unwrap_or_else(|_| Default::default())
    }
}

pub struct Menu {
    textures: HashMap<Texture, TextureHandle>,
    roms: HashMap<String, Rom>,
    sender: Sender<(String, Rom)>,
    receiver: Receiver<(String, Rom)>
}

impl Default for Menu {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            textures: HashMap::with_capacity(512),
            roms: HashMap::with_capacity(512),
            sender,
            receiver
        }
    }
}

impl Menu {
    fn search(&self, path: &String) {
        let sender = self.sender.clone();
        let walk = walkdir::WalkDir::new(path);
        std::thread::spawn(move || {
            for path in walk.max_depth(5).follow_links(true) {
                match path {
                    Ok(entry) => {
                        if !entry.file_type().is_file() { continue }
                        let ext = entry.path().extension().and_then(|x| x.to_str());
                        let key = entry.path().to_str();
                        if ext.is_none() || key.is_none() { continue };
                        if ["gbc", "gb"].contains(&ext.unwrap()) {
                            let key = key.unwrap().to_string();
                            if let Ok(rom) = Rom::load(entry.path()) {
                                sender.send((key, rom)).ok();
                            }
                        }
                    },
                    _ => {}
                }
            }
        });
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, conf: &mut RomConfig, path: P) {
        if let Some(path) = path.as_ref().to_str().map(|x| x.to_string()) {
            if conf.paths.contains(&path) { return }
            self.search(&path);
            conf.paths.push(path);
        }
    }

    fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

    fn add_cover(&mut self, rom_path: &String, rom: &mut Rom, ctx: &Context) {
        let dir = &rom.location;
        let path = PathBuf::from(rom_path);
        let mut names = vec![];
        for f in dir.read_dir().unwrap() {
            let f = if f.is_err() { continue; } else { f.unwrap() };
            let fpath = f.path();
            if fpath == path { continue ; }
            let ty = f.file_type();
            if ty.is_err() { continue; } else if !ty.unwrap().is_file() { continue }
            let ext = fpath.extension();
            let file = fpath.file_stem();
            if ext.is_some() && file.is_some() && ["jpeg", "jpg", "png"].contains(&fpath.extension().and_then(|x| x.to_str()).unwrap()) {
                if file.unwrap() == path.file_stem().unwrap() {
                    names.insert(0, fpath);
                    break ;
                }
                names.push(f.path());
            }
        }
        if !names.is_empty() {
            if let Some((tex, raw)) = ctx.load_image(rom_path, &names[0]) {
                self.textures.insert(Texture::Cover(rom_path.clone()), tex);
                rom.cover = Some(rom_path.clone());
                rom.raw = Some(raw);
            }
        }
    }
}

const ROM_GRID: f32 = 128.;

struct RomView<'a> {
    rom: &'a Rom,
    handle: Option<TextureHandle>
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
                let title = if title.chars().next().unwrap() == '\0' {
                    self.rom.filename.clone()
                } else {
                    title.clone()
                };
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
        for path in &emu.roms.paths {
            self.search(path);
            println!("looking at path {path}");
        }
    }

    fn draw(&mut self, ctx: &mut Context, emu: &mut Emulator) {
        while let Ok((path, mut rom)) = self.receiver.try_recv() {
            println!("found rom {:?} at {}", rom.header, path);
            if !self.roms.contains_key(&path) {
                self.add_cover(&path, &mut rom, ctx);
                self.roms.insert(path, rom);
            }
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
                    let [l, r] = match uis { [l, r] => [l, r], _ => unreachable!() };
                    let debug = egui::ImageButton::new(self.tex(Texture::Debug), (28., 28.)).frame(false);
                    let spritesheet = egui::ImageButton::new(self.tex(Texture::Spritesheet), (28., 28.)).frame(false);
                    let setting = egui::ImageButton::new(self.tex(Texture::Settings), (24., 24.)).frame(false);
                    let add = egui::ImageButton::new(self.tex(Texture::Add), (32., 32.)).frame(false);
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
                                if n as f32 * (ROM_GRID + ui.spacing().item_spacing.x * 2.) + ui.spacing().scroll_bar_width + ui.spacing().scroll_bar_outer_margin > w { ui.end_row(); n = 1; }
                                if ui.add(RomView::new(rom, &self.textures)).clicked() {
                                    // TODO defer full loading of rom to this point and just lazily fill the header during rom discovery
                                    emu.proxy.send_event(Events::Play(rom.clone())).ok();
                                }
                                n += 1;
                            }
                        });
                    });
            });
    }
}
