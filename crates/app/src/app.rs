use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

pub use config::{AppConfig, DbgConfig, RomConfig};
pub(crate) use render::{Shelf, ShelfItem};
use shared::egui::{Context, TextureHandle, TextureId};
use shared::rom::Rom;
use watcher::FileWatcher;

use crate::emulator::State;

mod config;
mod watcher;
mod render;

#[derive(Clone)]
pub enum Event<I> {
    Delete(PathBuf),
    Reload(PathBuf),
    Added(PathBuf, String, I),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Texture {
    Settings,
    Spritesheet,
    Debug,
    Add,
    Save,
    Nosave,
    SaveState,
    Cover(String),
}

pub(crate) struct Storage<Item: ShelfItem + 'static> {
    watcher: FileWatcher<Item>,
    sender: Sender<(PathBuf, String, Item)>,
    receiver: Receiver<(PathBuf, String, Item)>,
    pub shelves: Vec<Shelf<Item>>,
}

impl<I: ShelfItem + 'static> Storage<I> {
    fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            watcher: FileWatcher::new(),
            sender,
            receiver,
            shelves: Vec::with_capacity(128),
        }
    }

    fn new_root(&mut self, root: PathBuf, name: Option<String>) {
        self.watcher.add_path(root.clone());
        let shelf = Shelf::root(root.clone());
        self.shelves.push(if let Some(name) = name {
            shelf.with_name(name)
        } else {
            shelf
        });
        self.search(root);
    }

    fn search(&self, root: PathBuf) {
        let sender = self.sender.clone();
        let walk = walkdir::WalkDir::new(&root);
        std::thread::spawn(move || {
            for path in walk.max_depth(10).follow_links(true) {
                match path {
                    Ok(entry) => {
                        if !entry.file_type().is_file() { continue; }
                        let ext = entry.path().extension().and_then(|x| x.to_str());
                        let key = entry.path().to_str();
                        if ext.is_none() || key.is_none() { continue; };
                        if I::extensions().contains(&ext.unwrap()) {
                            let key = key.unwrap().to_string();
                            if let Ok(item) = I::load_from_path(entry.path()) {
                                sender.send((root.clone(), key, item)).ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    fn update(&mut self) -> Vec<Event<I>> {
        let mut events = self.watcher.iter().collect::<Vec<Event<I>>>();
        for evt in &events {
            match &evt {
                Event::Delete(path) => I::remove(self, path),
                Event::Reload(path) => {
                    if let Some(shelf) = self.shelves.iter_mut()
                        .find(|x| x.has_root(&path)) {
                        shelf.clear();
                    }
                    self.search(path.clone());
                }
                _ => unreachable!()
            }
        }
        while let Ok((root, path, item)) = self.receiver.try_recv() {
            events.push(Event::Added(root, path, item));
        }
        events
    }

    fn add_item(&mut self, root: PathBuf, path: String, item: I) {
        self.shelves.iter_mut().find(|x| x.has_root(&root))
            .map(|shelf| shelf.add(PathBuf::from(path), item))
            .unwrap_or_else(|| log::warn!("root shelf {root:?} does not exist !"));
    }
}

pub struct Menu {
    textures: HashMap<Texture, TextureHandle>,
    roms: Storage<Rom>,
    states: Storage<State>,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            textures: HashMap::with_capacity(512),
            roms: Storage::new(),
            states: Storage::new(),
        }
    }
}

impl Menu {
    pub fn add_path<P: AsRef<Path>>(&mut self, conf: &mut RomConfig, path: P) {
        let path = path.as_ref();
        let root = path.to_path_buf();
        if let Some(path) = path.to_str().map(|x| x.to_string()) {
            if conf.paths.contains(&path) { return; }
            self.roms.new_root(root, None);
            conf.paths.push(path);
        }
    }

    fn tex(&self, tex: Texture) -> TextureId {
        self.textures.get(&tex).unwrap().id()
    }

    fn add_cover<I: ShelfItem>(&mut self, item_path: &String, item: &mut I, ctx: &Context) {
        let id = Texture::Cover(item_path.clone());
        if self.textures.contains_key(&id) {
            item.set_cover(item_path.clone());
            return;
        }
        if let Some(tex) = item.load_cover(ctx) {
            self.textures.insert(id.clone(), tex);
            item.set_cover(item_path.clone());
        }
    }
}
