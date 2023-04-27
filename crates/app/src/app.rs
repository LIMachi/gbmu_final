use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

pub use config::{AppConfig, DbgConfig, RomConfig};
use shared::egui::{Context, TextureHandle, TextureId, Widget};
use shared::rom::Rom;
use shared::serde::{Deserialize, Serialize};
use shared::utils::image::ImageLoader;

use crate::app::watcher::FileWatcher;

mod config;
mod watcher;
mod render;


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum Texture {
    Settings,
    Spritesheet,
    Debug,
    Add,
    Save,
    Nosave,
    Cover(String),
}

pub struct Menu {
    textures: HashMap<Texture, TextureHandle>,
    roms: HashMap<String, Rom>,
    sender: Sender<(String, Rom)>,
    receiver: Receiver<(String, Rom)>,
    watcher: watcher::FileWatcher,
    shelves: HashMap<String, Vec<String>>,
}

impl Default for Menu {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            textures: HashMap::with_capacity(512),
            roms: HashMap::with_capacity(512),
            sender,
            receiver,
            watcher: FileWatcher::new(),
            shelves: Default::default(),
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
                        if !entry.file_type().is_file() { continue; }
                        let ext = entry.path().extension().and_then(|x| x.to_str());
                        let key = entry.path().to_str();
                        if ext.is_none() || key.is_none() { continue; };
                        if ["gbc", "gb"].contains(&ext.unwrap()) {
                            let key = key.unwrap().to_string();
                            if let Ok(rom) = Rom::load(entry.path()) {
                                sender.send((key, rom)).ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn add_rom(&mut self, path: String, mut rom: Rom, ctx: &mut Context) {
        if !self.roms.contains_key(&path) {
            self.add_cover(&path, &mut rom, ctx);
            self.roms.insert(path, rom);
        }
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, conf: &mut RomConfig, path: P) {
        if let Some(path) = path.as_ref().to_str().map(|x| x.to_string()) {
            if conf.paths.contains(&path) { return; }
            self.watcher.add_path(path.clone());
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
            if fpath == path { continue; }
            let ty = f.file_type();
            if ty.is_err() { continue; } else if !ty.unwrap().is_file() { continue; }
            let ext = fpath.extension();
            let file = fpath.file_stem();
            if ext.is_some() && file.is_some() && ["jpeg", "jpg", "png"].contains(&fpath.extension().and_then(|x| x.to_str()).unwrap()) {
                if file.unwrap() == path.file_stem().unwrap() {
                    names.insert(0, fpath);
                    break;
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
