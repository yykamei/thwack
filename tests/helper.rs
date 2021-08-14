use std::ascii::escape_default;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::fs::{create_dir_all, File};
use std::io;
use std::io::Write;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::Event;
use tempfile::{tempdir, TempDir};

use thwack::{Result, Terminal, TerminalEvent};

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

#[derive(Eq, PartialEq)]
pub struct Buffer {
    inner: Vec<u8>,
}

pub struct MockTerminal;

pub struct MockTerminalEvent {
    events: Arc<Mutex<VecDeque<Option<Event>>>>,
}

impl Buffer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    #[allow(dead_code)]
    pub fn normalize_path(mut self) -> Self {
        for value in self.inner.iter_mut() {
            if *value == b'\\' {
                *value = b'/';
            }
        }
        Self { inner: self.inner }
    }
}

impl Debug for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut buf: Vec<u8> = Vec::with_capacity(self.inner.len());
        &self.inner.iter().for_each(|v| {
            for e in escape_default(*v) {
                buf.push(e);
            }
        });
        let inner = from_utf8(&buf).unwrap();
        Debug::fmt(inner, f)
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

impl Terminal for MockTerminal {
    fn size(&self) -> Result<(u16, u16)> {
        Ok((80, 20))
    }

    fn enable_raw_mode(&self) -> Result<()> {
        Ok(())
    }

    fn disable_raw_mode(&self) -> Result<()> {
        Ok(())
    }
}

impl MockTerminalEvent {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    #[allow(dead_code)]
    pub fn add(&mut self, event: Option<Event>) {
        let data = self.events.clone();
        let mut events = data.lock().unwrap();
        events.push_back(event)
    }
}

impl TerminalEvent for MockTerminalEvent {
    fn poll(&self, _timeout: Duration) -> Result<bool> {
        let data = self.events.clone();
        let mut events = data.lock().unwrap();
        let event = events.front();
        if let Some(e) = event {
            match e {
                Some(_) => Ok(true),
                None => {
                    events.pop_front();
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    fn read(&mut self) -> Result<Event> {
        let e = self
            .events
            .clone()
            .lock()
            .unwrap()
            .pop_front()
            .unwrap()
            .unwrap();
        Ok(e)
    }
}

#[allow(dead_code)]
pub fn create_tree() -> io::Result<TempDir> {
    let tmp = tempdir()?;
    create_dir_all(tmp.path().join("src/a/b/c"))?;
    create_dir_all(tmp.path().join("src/Mo"))?;
    create_dir_all(tmp.path().join("lib/a/b/c"))?;
    create_dir_all(tmp.path().join("lib/x/y/z"))?;
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
    let _ = File::create(tmp.path().join("lib/df.js"))?;
    let _ = File::create(tmp.path().join("lib/ok.js"))?;
    let _ = File::create(tmp.path().join("lib/index.js"))?;
    let _ = File::create(tmp.path().join("lib/x/y/z/index.js"))?;
    let _ = File::create(tmp.path().join("lib/x/y/z/util.js"))?;
    let _ = File::create(tmp.path().join("lib/x/y/index.js"))?;
    let _ = File::create(tmp.path().join("lib/x/y/util.js"))?;
    let _ = File::create(tmp.path().join("lib/x/index.js"))?;
    let _ = File::create(tmp.path().join("lib/x/util.js"))?;
    let _ = File::create(tmp.path().join("package-lock.json"))?;
    let _ = File::create(tmp.path().join("package.json"))?;
    let _ = File::create(tmp.path().join("src/a/__test__.js"))?;
    let _ = File::create(tmp.path().join("src/a/b/c/index.js"))?;
    let _ = File::create(tmp.path().join("src/a/b/index.js"))?;
    let _ = File::create(tmp.path().join("src/a/index.js"))?;
    let _ = File::create(tmp.path().join("src/a/☕.js"))?;
    let _ = File::create(tmp.path().join("src/foo.js"))?;
    let _ = File::create(tmp.path().join("src/index.js"))?;
    let _ = File::create(tmp.path().join("src/Mo/index.js"))?;
    let _ = File::create(tmp.path().join("src/Mo/app.js"))?;
    let _ = File::create(tmp.path().join("src/Mo/utils.js"))?;
    let _ = File::create(tmp.path().join("src/Mo/demo.js"))?;
    let _ = File::create(tmp.path().join("src/Mo/hi.js"))?;
    let _ = File::create(tmp.path().join("tsconfig.json"))?;
    let _ = File::create(tmp.path().join("☕.txt"))?;
    Ok(tmp)
}
