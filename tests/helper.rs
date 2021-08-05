use std::fs::{create_dir_all, File};
use std::io;
use std::io::Write;

use tempfile::{tempdir, TempDir};

use thwack::{Result, Terminal};

#[macro_export]
macro_rules! buf {
    () => {{
        $crate::helper::Buffer::new()
    }};

    ($s:expr) => {{
        $crate::helper::Buffer::from($s)
    }};
}

#[macro_export]
macro_rules! args {
    ($($x:expr),+ $(,)?) => {
        vec![$(OsString::from($x)),+].into_iter()
    };
}

#[derive(Debug, Eq, PartialEq)]
pub struct Buffer {
    inner: Vec<u8>,
}

impl Buffer {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.inner.extend(buf);
        Ok(self.inner.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let _ = self.inner.extend(buf);
        Ok(())
    }
}

impl From<&str> for Buffer {
    fn from(s: &str) -> Self {
        Self {
            inner: Vec::from(s.as_bytes()),
        }
    }
}

impl From<String> for Buffer {
    fn from(s: String) -> Self {
        Self {
            inner: Vec::from(s.as_bytes()),
        }
    }
}

impl<const N: usize> From<&[u8; N]> for Buffer {
    fn from(b: &[u8; N]) -> Self {
        Self {
            inner: Vec::from(&b[..]),
        }
    }
}

pub struct MockTerminal;

impl Terminal for MockTerminal {
    fn size(&self) -> Result<(u16, u16)> {
        Ok((40, 40))
    }

    fn enable_raw_mode(&self) -> Result<()> {
        Ok(())
    }

    fn disable_raw_mode(&self) -> Result<()> {
        Ok(())
    }
}

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
