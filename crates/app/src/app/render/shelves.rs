use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use shared::egui::{Context, Grid, Response, RichText, TextureHandle, Ui, Widget};
use shared::egui::collapsing_header::CollapsingState;
use shared::Events;
use shared::rom::Rom;
use shared::utils::image::ImageLoader;

use crate::app::{Event, Storage, Texture};
use crate::emulator::Emulator;

use super::rom::ROM_GRID;
use super::rom::RomView;

pub(crate) trait ShelfItem: Ord + Send {
    fn render(&self, textures: &HashMap<Texture, TextureHandle>, ui: &mut Ui, events: &Sender<Event<Self>>) -> Response where Self: Sized;
    fn clicked(&self, shelf: &mut ShelfView<Self>) where Self: Sized;

    fn extensions() -> &'static [&'static str];

    fn load_from_path(path: &Path) -> std::io::Result<Self> where Self: Sized;

    fn set_cover(&mut self, cover: String);
    fn load_cover(&self, ctx: &Context) -> Option<TextureHandle>;
    fn remove(shelf: &mut Storage<Self>, path: &PathBuf) where Self: Sized;
}

impl ShelfItem for Rom {
    fn render(&self, textures: &HashMap<Texture, TextureHandle>, ui: &mut Ui, _: &Sender<Event<Self>>) -> Response {
        ui.add(RomView::new(self, textures))
    }

    fn clicked(&self, shelf: &mut ShelfView<Self>) {
        shelf.emu.proxy.send_event(Events::Play(self.clone())).ok();
    }

    fn extensions() -> &'static [&'static str] { &["gb", "gbc"] }

    fn load_from_path(path: &Path) -> std::io::Result<Self> where Self: Sized { Rom::load(path) }

    fn set_cover(&mut self, cover: String) {
        self.cover = Some(cover);
    }

    fn load_cover(&self, ctx: &Context) -> Option<TextureHandle> {
        let dir = &self.location;
        let path = dir.join(&self.filename);
        let stem = PathBuf::from(&self.filename).file_stem().unwrap().to_string_lossy().to_string();
        let mut default = None;
        dir.read_dir()
            .map_err(|e| log::warn!("could not read item directory {e:?}"))
            .ok()
            .and_then(|x|
                x.filter_map(|x| x.ok().filter(|e| e.file_type().map(|ty| ty.is_file()).unwrap_or(false)))
                    .filter(|e| e.path() != path)
                    .filter_map(|x|
                        x.path().extension().and_then(|ext| ext.to_str()).map(|s| s.to_string())
                            .and_then(|ext| {
                                if !["jpeg", "jpg", "png"].contains(&ext.as_str()) { return None; }
                                x.path().file_stem().and_then(|x| x.to_str()).map(|s| s.to_string())
                                    .map(|file| (x.path(), file))
                            })
                    )
                    .filter_map(|(path, file)| if file == "cover" || file == "default" {
                        default = Some(path);
                        None
                    } else { Some((path, file)) })
                    .find(|(_, x)| x.as_str() == stem)
                    .map(|(path, _)| path).or(default))
            .and_then(|x| ctx.load_image(&self.header.title, x))
            .map(|x| x.0)
    }

    fn remove(storage: &mut Storage<Rom>, path: &PathBuf) {
        storage.shelves.retain(|x| !x.has_root(path));
        storage.watcher.remove_path(path);
    }
}

pub(crate) struct Shelf<I: ShelfItem> {
    root: bool,
    name: Option<String>,
    pub(crate) path: PathBuf,
    cache: HashSet<PathBuf>,
    items: Vec<I>,
    subs: Vec<Shelf<I>>,
}

impl<Item: ShelfItem> Eq for Shelf<Item> {}

impl<Item: ShelfItem> PartialEq for Shelf<Item> {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl<Item: ShelfItem> PartialOrd for Shelf<Item> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl<Item: ShelfItem> Ord for Shelf<Item> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl<Item: ShelfItem> Shelf<Item> {
    pub(crate) fn view<'a>(&'a self, emu: &'a mut Emulator,
                           covers: &'a HashMap<Texture, TextureHandle>,
                           tx: Sender<Event<Item>>) -> ShelfView<Item> {
        ShelfView { shelf: self, covers, emu, tx, remove: true }
    }

    pub fn has_root<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path == path.as_ref()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.items.clear();
        self.subs.clear();
    }

    pub fn add(&mut self, path: PathBuf, item: Item) {
        let paths: Vec<PathBuf> = path.ancestors()
            .map(|x| x.to_path_buf())
            .collect();
        let root = self.path.clone();
        self.add_rec(&mut paths.iter().rev().skip_while(|x| x != &&root).skip(1).peekable(), item);
    }

    fn add_rec<'a, I: Iterator<Item=&'a PathBuf>>(&mut self, paths: &mut Peekable<I>, rom: Item) {
        let path = paths.next().expect("should at least be a file");
        if paths.peek().is_none() {
            if !self.cache.contains(path) {
                self.cache.insert(path.clone());
                self.items.push(rom);
                self.items.sort();
            }
        } else if let Some(sub) = self.subs.iter_mut().find(|x| x.has_root(path)) {
            sub.add_rec(paths, rom);
        } else {
            let mut shelf = Shelf::new(path.clone());
            shelf.add_rec(paths, rom);
            self.subs.push(shelf);
            self.subs.sort();
        }
    }

    pub fn new(path: PathBuf) -> Self {
        Self {
            root: false,
            path,
            items: vec![],
            subs: vec![],
            cache: Default::default(),
            name: None,
        }
    }

    pub fn root(path: PathBuf) -> Self {
        Self {
            root: true,
            path,
            name: None,
            items: vec![],
            subs: vec![],
            cache: Default::default(),
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}

pub(crate) struct ShelfView<'a, Item: ShelfItem> {
    covers: &'a HashMap<Texture, TextureHandle>,
    shelf: &'a Shelf<Item>,
    pub(crate) emu: &'a mut Emulator,
    tx: Sender<Event<Item>>,
    remove: bool,
}

impl<'a, Item: ShelfItem> ShelfView<'a, Item> {
    pub fn can_remove_root(mut self, remove: bool) -> Self {
        self.remove = remove;
        self
    }
}

impl<'a, Item: ShelfItem> Widget for ShelfView<'a, Item> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let path = if self.shelf.root {
            self.shelf.path.to_str().expect("path to string")
        } else {
            self.shelf.path.file_stem().and_then(|stem| stem.to_str()).expect("dir")
        };
        let id = ui.make_persistent_id("shelf_header_".to_owned() + path);

        CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label(RichText::new(self.shelf.name.as_ref().map(|e| e.as_str()).unwrap_or(path)).size(
                    if self.shelf.root { 24. } else { 16. }
                ));
                if self.shelf.root && self.remove && ui.button("-").clicked() {
                    self.tx.send(Event::Delete(self.shelf.path.clone())).ok();
                };
            })
            .body(|ui| {
                let w = ui.available_width();
                let mut res = Grid::new("shelf_grid_".to_owned() + path)
                    .show(ui, |ui| {
                        let mut n = 1;
                        for rom in &self.shelf.items {
                            if n as f32 * (ROM_GRID + ui.spacing().item_spacing.x * 1.) + ui.spacing().scroll_bar_width + ui.spacing().scroll_bar_outer_margin > w {
                                ui.end_row();
                                n = 1;
                            }
                            if rom.render(&self.covers, ui, &self.tx).clicked() { rom.clicked(&mut self); }
                            n += 1;
                        }
                    }).response;
                for shelf in &self.shelf.subs {
                    res |= ui.add(shelf.view(self.emu, self.covers, self.tx.clone()));
                }
                res
            }).0
    }
}

