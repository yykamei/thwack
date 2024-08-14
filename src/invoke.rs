use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::error::{Error, Result};
use crate::preferences::Preferences;

pub(crate) trait Execvp {
    fn execvp(&self, c: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int;
}

pub(crate) struct Libc;

impl Execvp for Libc {
    fn execvp(&self, c: *const libc::c_char, argv: *const *const libc::c_char) -> libc::c_int {
        unsafe { libc::execvp(c, argv) }
    }
}

/// Invoke the specified command with the selected path.
pub(crate) fn invoke(libc_impl: &dyn Execvp, preferences: &Preferences, path: &str) -> Result<()> {
    let mut cstrings: Vec<CString> = Vec::with_capacity(10); // TODO: Why is it 10?
    for arg in preferences.exec.split_whitespace() {
        cstrings.push(CString::new(arg)?);
    }
    cstrings.push(CString::new(path)?);
    let argv: Vec<*const c_char> = cstrings
        .iter()
        .map(|c| c.as_ptr())
        .chain(std::iter::once(ptr::null()))
        .collect();

    let errno = libc_impl.execvp(cstrings[0].as_ptr(), argv.as_ptr());

    if errno == 0 {
        return Ok(());
    }

    Err(Error::exec(&format!(
        "`{} {}` failed and returned {}",
        preferences.exec, path, errno
    )))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    struct MockLibc(libc::c_int);

    impl Execvp for MockLibc {
        fn execvp(
            &self,
            _c: *const libc::c_char,
            _argv: *const *const libc::c_char,
        ) -> libc::c_int {
            self.0
        }
    }

    #[test]
    fn test_invoke() {
        let preferences = Preferences {
            exec: String::from("echo"),
            ..Preferences::default()
        };
        let mock_libc = MockLibc(0);
        assert!(invoke(&mock_libc, &preferences, "Hello, world!").is_ok());
    }

    #[test]
    fn invoke_fail() {
        let preferences = Preferences {
            exec: String::from("non_existent_command"),
            ..Preferences::default()
        };
        let mock_libc = MockLibc(1);
        let result = invoke(&mock_libc, &preferences, "Hello, world!");
        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            "`non_existent_command Hello, world!` failed and returned 1"
        );
    }
}
