use crate::Result;
use git2::Repository;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct Tree {
    paths: Vec<String>,
}

impl Tree {
    pub(crate) fn new<P: AsRef<Path>>(dir: P, repo: Option<&Repository>) -> Result<Self> {
        let mut paths = Vec::new();
        extract_paths(&mut paths, dir, repo)?;
        Ok(Self { paths })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &String> {
        self.paths.iter()
    }
}

fn extract_paths<P: AsRef<Path>>(
    paths: &mut Vec<String>,
    current_dir: P,
    repo: Option<&Repository>,
) -> Result<()> {
    for entry in read_dir(current_dir.as_ref())? {
        let entry = entry?;
        let path = entry.path();
        if git_ignore(repo, &path) {
            continue;
        }
        // Use `DirEntry::file_type`, which does not follow symlinks, so that a
        // symlinked directory is not traversed. Traversing it could escape the
        // starting point or recurse forever on a symlink cycle.
        if entry.file_type()?.is_dir() {
            extract_paths(paths, &path, repo)?;
        } else if let Some(absolute) = path.to_str() {
            paths.push(absolute.to_string());
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
pub mod tests {
    use super::*;
    use git2::Signature;
    use std::fs::File;
    use std::fs::create_dir_all;
    use std::io;
    use std::io::Write;
    use tempfile::{TempDir, tempdir};

    pub fn create_files(with_git: bool) -> io::Result<TempDir> {
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
        if with_git {
            let repo = Repository::init(tmp.path()).unwrap();
            let signature = Signature::now("test", "test@example.com").unwrap();
            let tree = repo
                .find_tree(repo.index().unwrap().write_tree().unwrap())
                .unwrap();
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();
        }
        Ok(tmp)
    }

    #[test]
    fn new() {
        let dir = create_files(true).unwrap();
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(dir.path(), Some(&repo)).unwrap();
        for path in tree.iter() {
            assert!(path.starts_with(dir.path().to_str().unwrap()));
        }
    }

    #[cfg(unix)]
    #[test]
    fn does_not_recurse_into_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let dir = create_files(false).unwrap();
        let outside = tempdir().unwrap();
        let _ = File::create(outside.path().join("secret.txt")).unwrap();
        symlink(outside.path(), dir.path().join("link")).unwrap();

        let tree = Tree::new(dir.path(), None).unwrap();
        let paths: Vec<String> = tree.iter().cloned().collect();

        assert!(
            paths.iter().any(|p| p.ends_with("link")),
            "the symlink entry itself should be listed"
        );
        assert!(
            !paths.iter().any(|p| p.contains("secret.txt")),
            "files behind a symlinked directory must not be traversed"
        );
    }
}
