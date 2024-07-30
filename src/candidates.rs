use std::cmp::min;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

use git2::Repository;

use crate::matched_path::MatchedPath;
use crate::query::Query;
use crate::starting_point::StartingPoint;
use crate::Result;

pub(crate) struct Candidates {
    paths: Vec<MatchedPath>,
}

impl Candidates {
    fn new(
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
        Ok(Self { paths })
    }

    fn take(&self, n: usize) -> &[MatchedPath] {
        &self.paths[..min(n, self.paths.len())]
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
        let candidates = Candidates::new(&starting_point, &query, Some(&repo)).unwrap();
        let result = candidates.take(3);
        // TODO: This sort is not expected.
        assert_eq!(
            result.iter().map(|p| p.relative()).collect::<Vec<&str>>(),
            &[".config/bar.toml", ".config/ok.toml", ".env.local"]
        );
    }

    #[test]
    fn test_candidates_with_query() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("bar");
        let repo = Repository::open(dir.path()).unwrap();
        let candidates = Candidates::new(&starting_point, &query, Some(&repo)).unwrap();
        let result = candidates.take(5);
        // TODO: This sort is not expected.
        assert_eq!(
            result.iter().map(|p| p.relative()).collect::<Vec<&str>>(),
            &[".config/bar.toml", "lib/bar.js"]
        );
    }

    #[test]
    fn test_candidates_without_repo() {
        let dir = create_tree().unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let candidates = Candidates::new(&starting_point, &query, None).unwrap();
        let result: Vec<&str> = candidates.take(100).iter().map(|p| p.relative()).collect();
        assert!(result.contains(&"log.txt"));
        assert!(result.contains(&".git/config"));
    }
}
