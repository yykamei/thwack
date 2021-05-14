use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::fs::ReadDir;
use std::path::Path;

use crate::error::{Error, Result};

pub struct Finder {
    root: String,
    query: String,
    dirs: VecDeque<ReadDir>,
}

impl Finder {
    pub fn new(dir: impl AsRef<Path>, query: &str) -> Result<Self> {
        let root = dir.as_ref().canonicalize()?;
        let mut dirs = VecDeque::new();
        let read_dir = root.read_dir()?;
        dirs.push_back(read_dir);
        let root = root.to_str().ok_or(Error::invalid_unicode(
            "The passed directory does not seem to valid unicode.",
        ))?;
        Ok(Self {
            root: root.to_string(),
            query: query.to_string(),
            dirs,
        })
    }
}

impl Iterator for Finder {
    type Item = Result<MatchedPath>;

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
        assert!(path.is_absolute()); // TODO: Remove later
        if path.is_dir() {
            match path.read_dir() {
                Ok(dir) => {
                    self.dirs.push_back(dir);
                    self.next()
                }
                Err(e) => Some(Err(Error::from(e))),
            }
        } else {
            match MatchedPath::new(&self.query, &self.root, &path) {
                Some(matched) => Some(Ok(matched)),
                None => self.next(),
            }
        }
    }
}

#[derive(Debug)]
pub struct MatchedPath {
    absolute: String,
    relative: String,
    positions: Vec<usize>,
}

impl Display for MatchedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.relative, f)
    }
}

impl MatchedPath {
    pub(super) fn new(query: &str, root: &str, path: &Path) -> Option<Self> {
        let absolute = path.to_str()?;
        let relative = absolute
            .strip_prefix(root)
            .expect("The passed root must be prefix of the path.");
        let relative = &relative[1..]; // NOTE: Delete the prefix of slash
        let mut positions: Vec<usize> = vec![];
        for char in query.chars() {
            let begin = if let Some(pos) = positions.last() {
                pos + 1
            } else {
                0
            };
            // TODO: Explain this line later.
            let target = &relative[begin..].to_lowercase();
            let pos = target.find(char)?;
            positions.push(begin + pos);
        }
        Some(Self {
            absolute: absolute.to_string(),
            relative: relative.to_string(),
            positions,
        })
    }
    // TODO: Present with colorized value with emphasized positions.
}
