use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use shared::egui::{Grid, Response, TextureHandle, Ui, Widget};
use shared::egui::collapsing_header::CollapsingState;
use shared::Events;
use shared::rom::Rom;

use crate::app::{Event, Texture};
use crate::app::render::rom::RomView;
use crate::emulator::Emulator;

use super::rom::ROM_GRID;

pub(crate) trait ShelfItem {
    fn render(&self, textures: &HashMap<Texture, TextureHandle>, ui: &mut Ui) -> Response;
    fn clicked(&self, shelf: &ShelfView<Self>) where Self: Sized;
}

impl ShelfItem for Rom {
    fn render(&self, textures: &HashMap<Texture, TextureHandle>, ui: &mut Ui) -> Response {
        ui.add(RomView::new(self, textures))
    }

    fn clicked(&self, shelf: &ShelfView<Self>) {
        shelf.emu.proxy.send_event(Events::Play(self.clone())).ok();
    }
}

pub(crate) struct Shelf<I: ShelfItem> {
    root: bool,
    path: PathBuf,
    cache: HashSet<PathBuf>,
    roms: Vec<I>,
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

impl<Item: ShelfItem + Ord> Shelf<Item> {
    pub(crate) fn view<'a>(&'a self, emu: &'a mut Emulator,
                           covers: &'a HashMap<Texture, TextureHandle>,
                           tx: Sender<Event>) -> ShelfView<Item> {
        ShelfView { shelf: self, covers, emu, tx }
    }

    pub fn has_root<P: AsRef<Path>>(&self, path: P) -> bool {
        self.path == path.as_ref()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.roms.clear();
        self.subs.clear();
    }

    pub fn add(&mut self, path: PathBuf, rom: Item) {
        let paths: Vec<PathBuf> = path.ancestors()
            .map(|x| x.to_path_buf())
            .collect();
        let root = self.path.clone();
        self.add_rec(&mut paths.iter().rev().skip_while(|x| x != &&root).skip(1).peekable(), rom);
    }

    fn add_rec<'a, I: Iterator<Item=&'a PathBuf>>(&mut self, paths: &mut Peekable<I>, rom: Item) {
        let path = paths.next().expect("should at least be a file");
        if paths.peek().is_none() {
            if !self.cache.contains(path) {
                self.cache.insert(path.clone());
                self.roms.push(rom);
                self.roms.sort();
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
            roms: vec![],
            subs: vec![],
            cache: Default::default(),
        }
    }

    pub fn root(path: PathBuf) -> Self {
        Self {
            root: true,
            path,
            roms: vec![],
            subs: vec![],
            cache: Default::default(),
        }
    }
}

pub(crate) struct ShelfView<'a, Item: ShelfItem> {
    covers: &'a HashMap<Texture, TextureHandle>,
    shelf: &'a Shelf<Item>,
    emu: &'a mut Emulator,
    tx: Sender<Event>,
}

impl<'a, Item: ShelfItem + Ord> Widget for ShelfView<'a, Item> {
    fn ui(self, ui: &mut Ui) -> Response {
        let path = self.shelf.path.to_str().expect("path to string");
        let id = ui.make_persistent_id("shelf_header_".to_owned() + path);
        CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.label(path);
                if self.shelf.root && ui.button("-").clicked() {
                    self.tx.send(Event::Delete(self.shelf.path.clone())).ok();
                };
            })
            .body_unindented(|ui| {
                let w = ui.available_width();
                Grid::new("shelf_grid_".to_owned() + path)
                    .show(ui, |ui| {
                        let mut n = 1;
                        for rom in &self.shelf.roms {
                            if n as f32 * (ROM_GRID + ui.spacing().item_spacing.x * 1.) + ui.spacing().scroll_bar_width + ui.spacing().scroll_bar_outer_margin > w {
                                ui.end_row();
                                n = 1;
                            }
                            if rom.render(&self.covers, ui).clicked() { rom.clicked(&self); }
                            n += 1;
                        }
                    });
                for shelf in &self.shelf.subs {
                    ui.add(shelf.view(self.emu, self.covers, self.tx.clone()));
                }
            }).0
    }
}

