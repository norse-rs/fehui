

mod entry;
mod api;

use notify::Watcher;
use std::{error, fmt, io};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;
use entry::Entry;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Notify(notify::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Error {
        Error::Notify(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Notify(ref notify) => notify.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(match *self {
            Error::Io(ref err) => err,
            Error::Notify(ref notify) => notify,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(fmt),
            Error::Notify(ref err) => err.fmt(fmt),
        }
    }
}

pub struct Hotload {
    path: PathBuf,
    _watcher: notify::RecommendedWatcher,
    rx: Receiver<notify::DebouncedEvent>,
    entry: Option<entry::Entry>,
    old_entries: Vec<entry::Entry>, // needed for sth like dropping
}

impl Hotload {
    pub fn new<P: AsRef<Path>>(path: P, lib_name: &str) -> Result<Self, Error> {
        let (tx, rx) = channel();
        let path = path.as_ref().join(entry::dylib_name(lib_name));
        let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;


        Ok(Hotload {
            path,
            _watcher: watcher,
            rx,
            entry: None,
            old_entries: Vec::new(),
        })
    }

    pub fn is_loaded(&self) -> bool {
        self.entry.is_some()
    }

    pub fn try_reload(&mut self, ui: &mut fehui::FehUI) -> Result<bool, Error> {
        let mut reload = false;
        while let Ok(event) = self.rx.try_recv() {
            use notify::DebouncedEvent as Event;
            match event {
                Event::Create(_) | Event::Write(_) => reload = true,
                _ => (),
            }
        }

        if reload {
            self.reload(ui)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn reload(&mut self, ui: &mut fehui::FehUI) -> Result<(), Error> {
        if let Some(entry) = self.entry.take() {
            self.old_entries.push(entry);
        }
        self.entry = Some(Entry::new(&self.path)?);
        if let Some(ref mut entry) = &mut self.entry {
            entry.load(ui);
        }
        Ok(())
    }
}