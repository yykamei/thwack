use std::collections::VecDeque;
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub struct Finder {
    root: PathBuf,
    dirs: VecDeque<ReadDir>,
    query: String,
}

impl Finder {
    pub fn new(dir: impl AsRef<Path>, query: &str) -> Result<Self> {
        let root = dir.as_ref();
        let mut dirs = VecDeque::new();
        let read_dir = root.read_dir()?;
        dirs.push_back(read_dir);
        Ok(Self {
            root: root.to_path_buf(),
            dirs,
            query: query.to_string(),
        })
    }
}

impl Iterator for Finder {
    type Item = Result<PathBuf>; // TODO

    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.dirs.front_mut()?;
        let path = match dir.next() {
            Some(d) => d.map(|d| d.path()),
            None => {
                self.dirs
                    .pop_front()
                    .expect("`dirs` should have at least one item.");
                return self.next();
            }
        };
        let path = match path {
            Ok(path) => path,
            Err(e) => return Some(Err(Error::from(e))),
        };
        if path.is_dir() {
            match path.read_dir() {
                Ok(dir) => {
                    self.dirs.push_back(dir);
                    self.next()
                }
                Err(e) => Some(Err(Error::from(e))),
            }
        } else {
            // TODO: Filter with query
            Some(Ok(path))
        }
    }
}
