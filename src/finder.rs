use std::collections::VecDeque;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

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
                    )))
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
        for char in normalize_query(query).chars() {
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
}

fn canonicalize_starting_point(path: &Path) -> Result<PathBuf> {
    path.canonicalize().map_err(|_e|
        Error::args(&format!(
            "The specified starting point {:?} cannot be normalized. Perhaps, it might not exist or cannot be read.",
            path,
        ))
    )
}

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
