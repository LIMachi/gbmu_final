use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub enum Event {
    Reload(String),
}

pub struct FileWatcher {
    watchers: Vec<RecommendedWatcher>,
    tx: Sender<Event>,
    events: Receiver<Event>,
}

impl FileWatcher {
    pub fn new() -> Self {
        let (tx, events) = channel();
        Self { watchers: vec![], tx, events }
    }

    pub fn add_path(&mut self, path: String) {
        let x = path.clone();
        let tx = self.tx.clone();
        let mut w = notify::recommended_watcher(move |event| {
            match event {
                Ok(notify::Event {
                       kind: EventKind::Create(_) | EventKind::Remove(_),
                       ..
                   }) => { tx.send(Event::Reload(x.clone())).ok(); }
                _ => {}
            };
        }).expect("failed to build watcher");
        if w.watch(Path::new(&path), RecursiveMode::Recursive)
            .map_err(|e| log::warn!("failed to start watcher for path {path}, reason: {e:?}"))
            .is_ok()
        {
            self.watchers.push(w);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=Event> + '_ {
        self.events.try_iter()
    }
}
