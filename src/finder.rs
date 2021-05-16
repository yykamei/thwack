use std::collections::VecDeque;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::ReadDir;
use std::path::Path;

use crate::error::{Error, Result};

pub struct Finder {
    starting_point: String,
    query: String,
    dirs: VecDeque<ReadDir>,
}

impl Finder {
    pub fn new(starting_point: &Path, query: &str) -> Result<Self> {
        let mut dirs = VecDeque::new();
        let read_dir = starting_point.read_dir()?;
        dirs.push_back(read_dir);
        let starting_point = starting_point.to_str().ok_or_else(|| {
            Error::invalid_unicode("The passed directory does not seem to be valid unicode.")
        })?;
        Ok(Self {
            starting_point: starting_point.to_string(),
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
                    .expect("\"dirs\" should have at least one item.");
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
            assert!(path.is_absolute()); // TODO: Remove later
            let absolute = match path.to_str() {
                Some(path) => path,
                None => {
                    return Some(Err(Error::invalid_unicode(&format!(
                        "The path {:?} does not seem to be valid unicode.",
                        path
                    ))))
                }
            };
            match MatchedPath::new(&self.query, &self.starting_point, absolute) {
                Some(matched) => Some(Ok(matched)),
                None => self.next(),
            }
        }
    }
}

#[derive(Debug)]
pub struct MatchedPath {
    pub absolute: String,
    pub relative: String,
    pub positions: Vec<usize>,
}

impl Display for MatchedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.relative, f)
    }
}

impl MatchedPath {
    pub(super) fn new(query: &str, starting_point: &str, absolute: &str) -> Option<Self> {
        let relative = absolute
            .strip_prefix(starting_point)
            .expect("The passed starting_point must be prefix of the path.");
        let relative = &relative[1..]; // NOTE: Delete the prefix of slash
        let mut positions: Vec<usize> = vec![];
        for char in Self::normalize_query(query).chars() {
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

    #[cfg(target_os = "windows")]
    fn normalize_query(query: &str) -> String {
        // NOTE: Forward slashes are not allowed in a filename, so this replacing is supposed to work.
        //       See https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions
        query.replace('/', "\\")
    }

    #[cfg(not(target_os = "windows"))]
    fn normalize_query(query: &str) -> &str {
        query
    }
}
