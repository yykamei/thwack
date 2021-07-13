use std::io::{Result, Write};

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
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let _ = self.inner.extend(buf);
        Ok(self.inner.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let _ = self.inner.extend(buf);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
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

impl<const N: usize> From<&[u8; N]> for Buffer {
    fn from(b: &[u8; N]) -> Self {
        Self {
            inner: Vec::from(&b[..]),
        }
    }
}
