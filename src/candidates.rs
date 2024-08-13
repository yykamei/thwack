use std::fs::read_dir;
use std::path::{Path, PathBuf};

use git2::Repository;

use crate::matched_path::MatchedPath;
use crate::query::Query;
use crate::starting_point::StartingPoint;
use crate::Result;

#[derive(Debug)]
pub(crate) struct Candidates {
    paths: Vec<MatchedPath>,
    selected: Option<usize>,
}

impl Candidates {
    pub(crate) fn new(
        visible_paths_length: usize,
        starting_point: &StartingPoint,
        query: &Query,
        repo: Option<&Repository>,
    ) -> Result<Self> {
        let mut paths: Vec<MatchedPath> = Vec::new();
        extract_paths(
            &mut paths,
            starting_point,
            starting_point.as_ref(),
            query,
            repo,
        )?;
        paths.sort();
        paths.truncate(visible_paths_length);
        Ok(Self {
            paths,
            selected: None,
        })
    }

    pub(crate) fn selected(&self) -> Option<&MatchedPath> {
        if let Some(s) = self.selected {
            return self.paths.get(s);
        }
        None
    }

    pub(crate) fn move_down(&mut self) {
        let limit = self.paths.len();
        if limit == 0 {
            return;
        }
        if let Some(s) = self.selected {
            if s < limit - 1 {
                self.selected = Some(s + 1);
            }
        } else {
            self.selected = Some(0);
        }
    }

    pub(crate) fn move_up(&mut self) {
        let limit = self.paths.len();
        if limit == 0 {
            return;
        }
        if let Some(s) = self.selected {
            if s > 0 {
                self.selected = Some(s - 1);
            }
        }
    }
}

fn extract_paths<P: AsRef<Path>>(
    paths: &mut Vec<MatchedPath>,
    starting_point: &StartingPoint,
    current_dir: P,
    query: &Query,
    repo: Option<&Repository>,
) -> Result<()> {
    for entry in read_dir(current_dir.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if git_ignore(repo, &path) {
            continue;
        }
        if path.is_dir() {
            extract_paths(paths, starting_point, &path, query, repo)?;
        } else {
            if let Some(absolute) = path.to_str() {
                match MatchedPath::new(&query.to_string(), starting_point.as_ref(), absolute) {
                    Some(matched) => paths.push(matched),
                    None => continue,
                }
            }
        }
    }
    Ok(())
}

fn git_ignore(repo: Option<&Repository>, path: &PathBuf) -> bool {
    if let Some(r) = repo {
        match r.is_path_ignored(&path) {
            Ok(result) => {
                if result {
                    return true;
                }
            }
            Err(e) => {
                log::error!("is_path_ignored failed: {:?}", e);
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};
    use std::io;
    use std::io::Write;

    use git2::Signature;
    use tempfile::{tempdir, TempDir};

    use super::*;

    fn create_tree() -> io::Result<TempDir> {
        let tmp = tempdir()?;
        create_dir_all(tmp.path().join("src/a/b/c"))?;
        create_dir_all(tmp.path().join("lib/a/b/c"))?;
        create_dir_all(tmp.path().join(".config"))?;
        let _ = File::create(tmp.path().join(".gitignore"))?.write_all(b"log.txt")?;
        let _ = File::create(tmp.path().join("log.txt"))?;
        let _ = File::create(tmp.path().join(".browserslistrc"))?;
        let _ = File::create(tmp.path().join(".config/bar.toml"))?;
        let _ = File::create(tmp.path().join(".config/ok.toml"))?;
        let _ = File::create(tmp.path().join(".editorconfig"))?;
        let _ = File::create(tmp.path().join(".env"))?;
        let _ = File::create(tmp.path().join(".env.local"))?;
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
        // Prepare the Git repository
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

    #[test]
    fn test_candidates_without_query() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let candidates = Candidates::new(3, &starting_point, &query, Some(&repo)).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert_eq!(
            result,
            &[".browserslistrc", ".config/bar.toml", ".config/ok.toml"]
        );
    }

    #[test]
    fn test_candidates_with_query() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("bar");
        let repo = Repository::open(dir.path()).unwrap();
        let candidates = Candidates::new(5, &starting_point, &query, Some(&repo)).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert_eq!(result, &[".config/bar.toml", "lib/bar.js"]);
    }

    #[test]
    fn test_candidates_without_repo() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let candidates = Candidates::new(100, &starting_point, &query, None).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert!(result.contains(&"log.txt".to_string()));
        assert!(result.contains(&".git/config".to_string()));
    }

    #[test]
    fn test_move_down() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let mut candidates = Candidates::new(3, &starting_point, &query, Some(&repo)).unwrap();
        assert_eq!(candidates.selected, None);
        candidates.move_down();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(1));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));

        let mut candidates = Candidates::new(0, &starting_point, &query, Some(&repo)).unwrap();
        candidates.move_down();
        assert_eq!(candidates.selected, None);
        candidates.move_down();
        assert_eq!(candidates.selected, None);
    }

    #[test]
    fn test_move_up() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let mut candidates = Candidates::new(3, &starting_point, &query, Some(&repo)).unwrap();
        assert_eq!(candidates.selected, None);
        candidates.move_up();
        assert_eq!(candidates.selected, None);
        candidates.move_down();
        candidates.move_down();
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(1));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(0));

        let mut candidates = Candidates::new(0, &starting_point, &query, Some(&repo)).unwrap();
        candidates.move_up();
        assert_eq!(candidates.selected, None);
        candidates.move_up();
        assert_eq!(candidates.selected, None);
    }

    #[test]
    fn test_selected() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let mut candidates = Candidates::new(3, &starting_point, &query, Some(&repo)).unwrap();
        assert_eq!(candidates.selected(), None);

        candidates.move_down();
        assert_eq!(candidates.selected().unwrap().relative(), ".browserslistrc");

        candidates.move_down();
        assert!(candidates
            .selected()
            .unwrap()
            .relative()
            .ends_with("bar.toml"));

        candidates.move_up();
        assert_eq!(candidates.selected().unwrap().relative(), ".browserslistrc");
    }
}
