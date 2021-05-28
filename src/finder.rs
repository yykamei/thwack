use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::matched_path::MatchedPath;

pub(crate) struct Finder {
    starting_point: String,
    query: String,
    dirs: VecDeque<ConsumedDir>,
}

impl Finder {
    pub(crate) fn new(starting_point: &str, query: &str) -> Result<Self> {
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

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};
    use std::io;

    use tempfile::{tempdir, TempDir};

    use super::*;

    pub fn create_tree() -> io::Result<TempDir> {
        let tmp = tempdir()?;
        create_dir_all(tmp.path().join("src/a/b/c"))?;
        create_dir_all(tmp.path().join("lib/a/b/c"))?;
        create_dir_all(tmp.path().join(".config"))?;
        let _ = File::create(tmp.path().join(".browserslistrc"))?;
        let _ = File::create(tmp.path().join(".config/bar.toml"))?;
        let _ = File::create(tmp.path().join(".config/ok.toml"))?;
        let _ = File::create(tmp.path().join(".editorconfig"))?;
        let _ = File::create(tmp.path().join(".env"))?;
        let _ = File::create(tmp.path().join(".env.local"))?;
        let _ = File::create(tmp.path().join(".gitignore"))?;
        let _ = File::create(tmp.path().join(".npmrc"))?;
        let _ = File::create(tmp.path().join(".nvmrc"))?;
        let _ = File::create(tmp.path().join("Dockerfile"))?;
        let _ = File::create(tmp.path().join("LICENSE"))?;
        let _ = File::create(tmp.path().join("README.md"))?;
        let _ = File::create(tmp.path().join("lib/a/b/c/index.js"))?;
        let _ = File::create(tmp.path().join("lib/a/b/c/☕.js"))?;
        let _ = File::create(tmp.path().join("lib/a/b/index.js"))?;
        let _ = File::create(tmp.path().join("lib/a/index.js"))?;
        let _ = File::create(tmp.path().join("lib/bar.js"))?;
        let _ = File::create(tmp.path().join("lib/index.js"))?;
        let _ = File::create(tmp.path().join("package-lock.json"))?;
        let _ = File::create(tmp.path().join("package.json"))?;
        let _ = File::create(tmp.path().join("src/a/__test__.js"))?;
        let _ = File::create(tmp.path().join("src/a/b/c/index.js"))?;
        let _ = File::create(tmp.path().join("src/a/b/index.js"))?;
        let _ = File::create(tmp.path().join("src/a/index.js"))?;
        let _ = File::create(tmp.path().join("src/a/☕.js"))?;
        let _ = File::create(tmp.path().join("src/foo.js"))?;
        let _ = File::create(tmp.path().join("src/index.js"))?;
        let _ = File::create(tmp.path().join("tsconfig.json"))?;
        let _ = File::create(tmp.path().join("☕.txt"))?;
        Ok(tmp)
    }

    fn find_paths(starting_point: &str, query: &str) -> Vec<String> {
        let mut paths = vec![];
        for matched in Finder::new(starting_point, query).unwrap() {
            let path = matched.unwrap();
            paths.push(path);
        }
        let mut paths: Vec<String> = paths
            .iter()
            .map(|m| m.relative().replace('\\', "/"))
            .collect();
        paths.sort();
        paths
    }

    #[test]
    fn returns_all_paths() {
        let dir = create_tree().unwrap();
        let size = find_paths(dir.path().to_str().unwrap(), "").len();
        assert_eq!(size, 29);
    }

    #[test]
    fn returns_empty() {
        let dir = create_tree().unwrap();
        let size = find_paths(
            dir.path().to_str().unwrap(),
            "the word should be not found with 🎂",
        )
            .len();
        assert_eq!(size, 0);
    }

    #[test]
    fn returns_filtered_paths_with_only_separator() {
        let dir = create_tree().unwrap();
        let size = find_paths(dir.path().to_str().unwrap(), "/").len();
        assert_eq!(size, 15);
    }

    #[test]
    fn returns_filtered_paths_with_uppercase() {
        let dir = create_tree().unwrap();
        let paths = find_paths(dir.path().to_str().unwrap(), "licenSE");
        assert_eq!(paths.len(), 1);
        assert_eq!(paths, vec!["LICENSE"]);
    }

    #[test]
    fn returns_filtered_paths_with_emoji_coffee() {
        let dir = create_tree().unwrap();
        let paths = find_paths(dir.path().to_str().unwrap(), "☕");
        assert_eq!(paths.len(), 3);
        assert_eq!(paths, vec!["lib/a/b/c/☕.js", "src/a/☕.js", "☕.txt"]);
    }
}
