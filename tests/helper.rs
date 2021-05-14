use std::fs::{create_dir_all, File};
use std::io;
use std::path::PathBuf;

use tempfile::{tempdir, TempDir};

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
    let _ = File::create(tmp.path().join("src/a/☕️.js"))?;
    let _ = File::create(tmp.path().join("src/foo.js"))?;
    let _ = File::create(tmp.path().join("src/index.js"))?;
    let _ = File::create(tmp.path().join("tsconfig.json"))?;
    Ok(tmp)
}
