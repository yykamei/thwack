use std::collections::VecDeque;
use std::path::Path;

use git2::Repository;

use crate::error::{Error, Result};
use crate::matched_path::MatchedPath;
use crate::starting_point::StartingPoint;

pub(crate) struct Finder<'a> {
    starting_point: &'a StartingPoint,
    query: &'a str,
    dirs: VecDeque<ConsumedDir>,
    repo: Option<&'a Repository>,
}

impl<'a> Finder<'a> {
    pub(crate) fn new(
        starting_point: &'a StartingPoint,
        query: &'a str,
        repo: Option<&'a Repository>,
    ) -> Result<Self> {
        let mut dirs = VecDeque::with_capacity(100);
        let consumed_dir = ConsumedDir::new(starting_point.as_ref(), repo)?;
        dirs.push_back(consumed_dir);
        Ok(Self {
            starting_point,
            query,
            dirs,
            repo,
        })
    }
}

impl<'a> Iterator for Finder<'a> {
    type Item = Result<MatchedPath>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let dir = self.dirs.front_mut()?;
            let absolute = match dir.next() {
                Some(Entry::File(path)) => path,
                Some(Entry::Dir(path)) => match ConsumedDir::new(&path, self.repo) {
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
            match MatchedPath::new(self.query, self.starting_point.as_ref(), &absolute) {
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
    entries: VecDeque<Entry>,
}

impl ConsumedDir {
    /// `ConsumedDir::new` assumes the passed `root` has already been canonicalized.
    fn new(root: &str, repo: Option<&Repository>) -> Result<Self> {
        let mut entries = VecDeque::with_capacity(50);
        let dir: &Path = root.as_ref();
        for de in dir.read_dir()? {
            let path = de?.path();
            if let Some(r) = repo {
                match r.is_path_ignored(&path) {
                    Ok(result) => {
                        if result {
                            continue;
                        }
                    }
                    Err(e) => {
                        log::error!("is_path_ignored failed: {:?}", e);
                        continue;
                    }
                }
            }
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
        Ok(Self { entries })
    }
}

impl Iterator for ConsumedDir {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        self.entries.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};
    use std::io;
    use std::io::Write;

    use git2::{Repository, Signature};
    use tempfile::{tempdir, TempDir};

    use super::*;

    fn create_tree() -> io::Result<TempDir> {
        let tmp = tempdir()?;
        create_dir_all(tmp.path().join("src/a/b/c"))?;
        create_dir_all(tmp.path().join("lib/a/b/c"))?;
        create_dir_all(tmp.path().join(".config"))?;
        let _ = File::create(tmp.path().join(".log.txt"))?;
        let _ = File::create(tmp.path().join(".browserslistrc"))?;
        let _ = File::create(tmp.path().join(".config/bar.toml"))?;
        let _ = File::create(tmp.path().join(".config/ok.toml"))?;
        let _ = File::create(tmp.path().join(".editorconfig"))?;
        let _ = File::create(tmp.path().join(".env"))?;
        let _ = File::create(tmp.path().join(".env.local"))?;
        let _ = File::create(tmp.path().join(".gitignore"))?.write_all(b".log.txt")?;
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
        let repo = Repository::init(tmp.path()).unwrap();
        let signature = Signature::now("test", "test@example.com").unwrap();
        let tree = repo
            .find_tree(repo.index().unwrap().write_tree().unwrap())
            .unwrap();
        let _ = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();
        Ok(tmp)
    }

    fn find_paths(starting_point: &str, query: &str, repo: Option<&Repository>) -> Vec<String> {
        let starting_point = StartingPoint::new(starting_point).unwrap();
        let mut paths = vec![];
        for matched in Finder::new(&starting_point, query, repo).unwrap() {
            let path = matched.unwrap();
            paths.push(path);
        }
        let mut paths: Vec<String> = paths
            .iter()
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        paths.sort();
        paths
    }

    #[test]
    fn returns_all_paths() {
        let dir = create_tree().unwrap();
        let paths = find_paths(dir.path().to_str().unwrap(), "", None);
        assert_eq!(paths.len(), 40);
        assert!(paths.contains(&".git/config".to_string()));
        assert!(paths.contains(&".log.txt".to_string()));
    }

    #[test]
    fn returns_all_paths_excluding_git() {
        let dir = create_tree().unwrap();
        let repo = Some(Repository::open(dir.path()).unwrap());
        let paths = find_paths(dir.path().to_str().unwrap(), "", repo.as_ref());
        assert_eq!(paths.len(), 29);
        assert!(!paths.contains(&".git/config".to_string()));
        assert!(!paths.contains(&".log.txt".to_string()));
    }

    #[test]
    fn returns_empty() {
        let dir = create_tree().unwrap();
        let repo = Some(Repository::open(dir.path()).unwrap());
        let size = find_paths(
            dir.path().to_str().unwrap(),
            "the word should be not found with 🎂",
            repo.as_ref(),
        )
        .len();
        assert_eq!(size, 0);
    }

    #[test]
    fn returns_filtered_paths_with_only_separator() {
        let dir = create_tree().unwrap();
        let repo = Some(Repository::open(dir.path()).unwrap());
        let size = find_paths(dir.path().to_str().unwrap(), "/", repo.as_ref()).len();
        assert_eq!(size, 15);
    }

    #[test]
    fn returns_filtered_paths_with_uppercase() {
        let dir = create_tree().unwrap();
        let repo = Some(Repository::open(dir.path()).unwrap());
        let paths = find_paths(dir.path().to_str().unwrap(), "licenSE", repo.as_ref());
        assert_eq!(paths.len(), 1);
        assert_eq!(paths, vec!["LICENSE"]);
    }

    #[test]
    fn returns_filtered_paths_with_emoji_coffee() {
        let dir = create_tree().unwrap();
        let repo = Some(Repository::open(dir.path()).unwrap());
        let paths = find_paths(dir.path().to_str().unwrap(), "☕", repo.as_ref());
        assert_eq!(paths.len(), 3);
        assert_eq!(paths, vec!["lib/a/b/c/☕.js", "src/a/☕.js", "☕.txt"]);
    }
}
