use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

pub use config::{AppConfig, DbgConfig, RomConfig};
use render::Shelf;
use shared::egui::{Context, TextureHandle, TextureId, Widget};
use shared::rom::Rom;
use shared::serde::{Deserialize, Serialize};
use shared::utils::image::{ImageLoader, RawData};
use watcher::FileWatcher;

mod config;
mod watcher;
mod render;

pub enum Event {
    Delete(PathBuf),
    Reload(String),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Texture {
    Settings,
    Spritesheet,
    Debug,
    Add,
    Save,
    Nosave,
    Cover(String),
}

pub struct Menu {
    roms: HashSet<String>,
    raw_tex: HashMap<Texture, RawData>,
    textures: HashMap<Texture, TextureHandle>,
    sender: Sender<(PathBuf, String, Rom)>,
    receiver: Receiver<(PathBuf, String, Rom)>,
    watcher: FileWatcher,
    shelves: Vec<Shelf>,
}

impl Default for Menu {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self {
            roms: HashSet::with_capacity(512),
            textures: HashMap::with_capacity(512),
            raw_tex: HashMap::with_capacity(512),
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
        let root = PathBuf::from(path);
        std::thread::spawn(move || {
            for path in walk.max_depth(10).follow_links(true) {
                match path {
                    Ok(entry) => {
                        if !entry.file_type().is_file() { continue; }
                        let ext = entry.path().extension().and_then(|x| x.to_str());
                        let key = entry.path().to_str();
                        if ext.is_none() || key.is_none() { continue; };
                        if ["gbc", "gb"].contains(&ext.unwrap()) {
                            let key = key.unwrap().to_string();
                            if let Ok(rom) = Rom::load(entry.path()) {
                                sender.send((root.clone(), key, rom)).ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    pub fn add_rom(&mut self, root: PathBuf, path: String, mut rom: Rom, ctx: &mut Context) {
        self.add_cover(&path, &mut rom, ctx);
        self.shelves.iter_mut().find(|x| x.has_root(&root))
            .map(|x| {
                x.add(PathBuf::from(path.clone()), rom)
            })
            .unwrap_or_else(|| {
                self.roms.remove(&path);
                log::warn!("root shelf {root:?} does not exist !");
            })
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, conf: &mut RomConfig, path: P) {
        let path = path.as_ref();
        let root = path.to_path_buf();
        if let Some(path) = path.to_str().map(|x| x.to_string()) {
            if conf.paths.contains(&path) { return; }
            self.shelves.push(Shelf::root(root));
            self.watcher.add_path(path.clone());
            self.search(&path);
            conf.paths.push(path);
        }
    }

    fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

    fn add_cover(&mut self, rom_path: &String, rom: &mut Rom, ctx: &Context) {
        let id = Texture::Cover(rom_path.clone());
        if let Some(raw) = self.raw_tex.get(&id) {
            rom.cover = Some(rom_path.clone());
            rom.raw = Some(raw.clone());
            return;
        }
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
            if ext.is_some() && file.is_some() && ["jpeg", "jpg", "png"].contains(&ext.and_then(|x| x.to_str()).unwrap()) {
                let file = file.unwrap();
                if file == path.file_stem().unwrap() {
                    names.insert(0, fpath);
                    break;
                }
                if file == "cover" || file == "default" {
                    names.push(f.path());
                }
            }
        }
        if !names.is_empty() {
            if let Some((tex, raw)) = ctx.load_image(rom_path, &names[0]) {
                self.textures.insert(id.clone(), tex);
                self.raw_tex.insert(id, raw.clone());
                rom.cover = Some(rom_path.clone());
                rom.raw = Some(raw);
            }
        }
    }
}
