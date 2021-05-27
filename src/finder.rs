use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::matched_path::MatchedPath;

pub struct Finder {
    starting_point: String,
    query: String,
    dirs: VecDeque<ConsumedDir>,
}

impl Finder {
    pub fn new(starting_point: &str, query: &str) -> Result<Self> {
        let mut dirs = VecDeque::with_capacity(100);
        let consumed_dir = ConsumedDir::new(starting_point)?;
        let starting_point = consumed_dir.root.clone();
        dirs.push_back(consumed_dir);
        Ok(Self {
            starting_point,
            query: query.to_string(),
            dirs,
        })
    }
}

impl Iterator for Finder {
    type Item = Result<MatchedPath>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.front_mut()?;
            let absolute = match dir.next() {
                Some(Entry::File(path)) => path,
                Some(Entry::Dir(path)) => match ConsumedDir::new(&path) {
                    Ok(c) => {
                        self.dirs.push_back(c);
                        continue;
                    }
                    Err(error) => return Some(Err(error)),
                },
                None => {
                    self.dirs
                        .pop_front()
                        .expect("\"dirs\" should have at least one item.");
                    continue;
                }
            };
            match MatchedPath::new(&self.query, &self.starting_point, &absolute) {
                Some(matched) => return Some(Ok(matched)),
                None => continue,
            }
        }
    }
}

enum Entry {
    File(String),
    Dir(String),
}

struct ConsumedDir {
    root: String,
    entries: VecDeque<Entry>,
}

impl ConsumedDir {
    fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let mut entries = VecDeque::with_capacity(50);
        let dir = canonicalize_starting_point(dir.as_ref())?;
        for de in dir.read_dir()? {
            let path = de?.path();
            let path_string = match path.to_str() {
                Some(path) => path.to_string(),
                None => {
                    return Err(Error::invalid_unicode(&format!(
                        "The path {:?} does not seem to be valid unicode.",
                        path
                    )));
                }
            };
            // TODO: Symbolic link?
            if path.is_dir() {
                entries.push_back(Entry::Dir(path_string));
            } else {
                entries.push_back(Entry::File(path_string));
            }
        }
        let root = dir
            .to_str()
            .expect("This function has already received \"dir\" as &str, so this is supposed to be valid unicode.")
            .to_string();
        Ok(Self { root, entries })
    }
}

impl Iterator for ConsumedDir {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop_front()
    }
}

fn canonicalize_starting_point(path: &Path) -> Result<PathBuf> {
    path.canonicalize().map_err(|_e|
        Error::args(&format!(
            "The specified starting point {:?} cannot be normalized. Perhaps, it might not exist or cannot be read.",
            path,
        ))
    )
}
