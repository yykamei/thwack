use std::collections::VecDeque;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::ReadDir;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub struct Finder {
    starting_point: String,
    query: String,
    dirs: VecDeque<ReadDir>,
}

impl Finder {
    pub fn new(starting_point: &str, query: &str) -> Result<Self> {
        let mut dirs = VecDeque::new();
        let starting_point: &Path = starting_point.as_ref();
        let starting_point = Self::canonicalize_starting_point(starting_point)?;
        let read_dir = starting_point.read_dir()?;
        dirs.push_back(read_dir);
        Ok(Self {
            starting_point: starting_point
                .to_str()
                .expect("This function has already received \"starting_point\" as &str, so this is supposed to be valid unicode.")
                .to_string(),
            query: query.to_string(),
            dirs,
        })
    }

    fn canonicalize_starting_point(path: &Path) -> Result<PathBuf> {
        path.canonicalize().map_err(|_e|
            Error::args(&format!(
                "The specified starting point {:?} cannot be normalized. Perhaps, it might not exist or cannot be read.",
                path,
            ))
        )
    }
}

impl Iterator for Finder {
    type Item = Result<MatchedPath>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.front_mut()?;
            let path = match dir.next() {
                Some(d) => d.map(|d| d.path()),
                None => {
                    self.dirs
                        .pop_front()
                        .expect("\"dirs\" should have at least one item.");
                    continue;
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
                        continue;
                    }
                    Err(e) => return Some(Err(Error::from(e))),
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
                    Some(matched) => return Some(Ok(matched)),
                    None => continue,
                }
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
        let relative: Vec<char> = absolute
            .strip_prefix(starting_point)
            .expect("The passed starting_point must be prefix of the path.")
            .chars()
            .collect();
        let relative = &relative[1..]; // NOTE: Delete the prefix of slash
        let mut positions: Vec<usize> = vec![];
        for char in Self::normalize_query(query).chars() {
            let begin = if let Some(pos) = positions.last() {
                pos + 1
            } else {
                0
            };
            // TODO: Explain this line later.
            let target = &relative[begin..];
            let pos = target.iter().position(|t| char.eq_ignore_ascii_case(t))?;
            positions.push(begin + pos);
        }
        Some(Self {
            absolute: absolute.to_string(),
            relative: relative.iter().collect(),
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
