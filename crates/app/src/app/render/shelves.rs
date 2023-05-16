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

#[derive(Eq)]
pub struct Shelf {
    root: bool,
    path: PathBuf,
    cache: HashSet<PathBuf>,
    roms: Vec<Rom>,
    subs: Vec<Shelf>,
}

impl PartialEq for Shelf {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl PartialOrd for Shelf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl Ord for Shelf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl Shelf {
    pub(crate) fn view<'a>(&'a mut self, emu: &'a mut Emulator,
                           covers: &'a HashMap<Texture, TextureHandle>,
                           tx: Sender<Event>) -> ShelfView {
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

    pub fn add(&mut self, path: PathBuf, rom: Rom) {
        let paths: Vec<PathBuf> = path.ancestors()
            .map(|x| x.to_path_buf())
            .collect();
        let root = self.path.clone();
        self.add_rec(&mut paths.iter().rev().skip_while(|x| x != &&root).skip(1).peekable(), rom);
    }

    fn add_rec<'a, I: Iterator<Item=&'a PathBuf>>(&mut self, paths: &mut Peekable<I>, rom: Rom) {
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

pub struct ShelfView<'a> {
    covers: &'a HashMap<Texture, TextureHandle>,
    shelf: &'a mut Shelf,
    emu: &'a mut Emulator,
    tx: Sender<Event>,
}

impl<'a> Widget for ShelfView<'a> {
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
                            if ui.add(RomView::new(rom, &self.covers)).clicked() {
                                self.emu.proxy.send_event(Events::Play(rom.clone())).ok();
                            }
                            n += 1;
                        }
                    });
                for shelf in &mut self.shelf.subs {
                    ui.add(shelf.view(self.emu, self.covers, self.tx.clone()));
                }
            }).0
    }
}

