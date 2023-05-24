use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::Event;

pub struct FileWatcher<Item> {
    watchers: HashMap<PathBuf, RecommendedWatcher>,
    tx: Sender<Event<Item>>,
    events: Receiver<Event<Item>>,
}

impl<Item: Send + 'static> FileWatcher<Item> {
    pub fn new() -> Self {
        let (tx, events) = channel();
        Self { watchers: Default::default(), tx, events }
    }

    pub fn add_path(&mut self, path: PathBuf) {
        let x = path.clone();
        let tx = self.tx.clone();
        let mut w = notify::recommended_watcher(move |event| {
            match event {
                Ok(e @ notify::Event {
                    kind: EventKind::Create(_) | EventKind::Remove(_) | EventKind::Modify(_),
                    ..
                }) => {
                    log::debug!("{e:?}");
                    tx.send(Event::Reload(x.clone())).ok();
                }
                _ => {}
            };
        }).expect("failed to build watcher");
        if w.watch(Path::new(&path), RecursiveMode::Recursive)
            .map_err(|e| log::warn!("failed to start watcher for path {path:?}, reason: {e:?}"))
            .is_ok()
        {
            self.watchers.insert(PathBuf::from(path), w);
        }
    }

    pub fn remove_path(&mut self, buf: &PathBuf) {
        self.watchers.remove(buf);
    }

    pub fn iter(&self) -> impl Iterator<Item=Event<Item>> + '_ {
        self.events.try_iter()
    }

    pub fn tx(&self) -> Sender<Event<Item>> { self.tx.clone() }
}
