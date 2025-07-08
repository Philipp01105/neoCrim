use crate::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
    watched_paths: HashMap<std::path::PathBuf, ()>,
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let (sender, receiver): (
            Sender<notify::Result<Event>>,
            Receiver<notify::Result<Event>>,
        ) = mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sender.send(res);
            },
            notify::Config::default(),
        )?;

        Ok(Self {
            watcher,
            receiver,
            watched_paths: HashMap::new(),
        })
    }

    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        if !self.watched_paths.contains_key(&path) {
            self.watcher.watch(&path, RecursiveMode::NonRecursive)?;
            self.watched_paths.insert(path, ());
        }

        Ok(())
    }

    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        if self.watched_paths.contains_key(&path) {
            self.watcher.unwatch(&path)?;
            self.watched_paths.remove(&path);
        }

        Ok(())
    }

    pub fn poll_events(&mut self) -> Vec<FileEvent> {
        let mut events = Vec::new();

        while let Ok(result) = self.receiver.try_recv() {
            match result {
                Ok(event) => {
                    for path in event.paths {
                        match event.kind {
                            EventKind::Modify(_) => {
                                events.push(FileEvent::Modified(path));
                            }
                            EventKind::Create(_) => {
                                events.push(FileEvent::Created(path));
                            }
                            EventKind::Remove(_) => {
                                events.push(FileEvent::Deleted(path));
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    log::error!("File watcher error: {e}");
                }
            }
        }

        events
    }
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    Modified(std::path::PathBuf),
    Created(std::path::PathBuf),
    Deleted(std::path::PathBuf),
    Renamed(std::path::PathBuf, std::path::PathBuf),
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            log::error!("Failed to create file watcher, using dummy implementation");
            Self {
                watcher: RecommendedWatcher::new(|_| {}, notify::Config::default()).unwrap(),
                receiver: mpsc::channel().1,
                watched_paths: HashMap::new(),
            }
        })
    }
}
