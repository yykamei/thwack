use std::env;
use std::ffi::OsString;
use std::io::{self, Stderr, Stdout, Write};
use std::process::exit;

use crate::args::{Args, HELP};
use crate::error::Result;
use crate::logger;
use crate::screen::Screen;
use crate::terminal::Terminal;

pub fn safe_exit(code: i32, stdout: Stdout, stderr: Stderr) {
    let _ = stdout.lock().flush();
    let _ = stderr.lock().flush();
    exit(code)
}

pub fn entrypoint<A: Iterator<Item = OsString>, W: Write, T: Terminal>(
    args: A,
    terminal: T,
    stdout: &mut W,
) -> Result<()> {
    let preferences = Args::new(args, env::vars_os()).parse()?;

    if let Some(ref path) = preferences.log_file {
        logger::init(path)?;
        log::debug!("Logger initialized!");
    }
    if preferences.help {
        print_and_flush(stdout, HELP)?;
        log::debug!("Show help and exit");
        return Ok(());
    }
    if preferences.version {
        print_and_flush(
            stdout,
            &format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )?;
        log::debug!("Show version and exit");
        return Ok(());
    }

    Screen::new(&preferences, &terminal, stdout)?.start()
}

fn print_and_flush(buffer: &mut impl Write, content: &str) -> io::Result<()> {
    buffer.write_all(content.as_bytes())?;
    buffer.flush()
}

#[cfg(test)]
mod tests {
    use crate::DefaultTerminal;

    use super::*;

    macro_rules! args {
        ($($x:expr),+ $(,)?) => {
            vec![$(OsString::from($x)),+].into_iter()
        };
    }

    #[derive(Eq, PartialEq)]
    pub struct Buffer(Vec<u8>);

    impl Write for Buffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let _ = self.0.extend(buf);
            Ok(self.0.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            let _ = self.0.extend(buf);
            Ok(())
        }
    }

    #[test]
    fn show_help() {
        let args = args!["thwack", "--help"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.0, HELP.as_bytes().to_vec());
    }

    #[test]
    fn show_help_with_version() {
        let args = args!["thwack", "--help", "--version"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.0, HELP.as_bytes().to_vec());
    }

    #[test]
    fn show_help_with_query() {
        let args = args!["thwack", "--help", "--", "query"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.0, HELP.as_bytes().to_vec());
    }

    #[test]
    fn show_help_with_starting_point() {
        let args = args!["thwack", "--help", "--starting-point=/tmp"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(buffer.0, HELP.as_bytes().to_vec());
    }

    #[test]
    fn show_version() {
        let args = args!["thwack", "--version"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(
            buffer.0,
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_bytes()
        );
    }

    #[test]
    fn show_version_with_query() {
        let args = args!["thwack", "--version", "--", "query"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(
            buffer.0,
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_bytes()
        );
    }

    #[test]
    fn show_version_with_starting_point() {
        let args = args!["thwack", "--version", "--starting-point=/tmp"];
        let mut buffer = Buffer(vec![]);
        let result = entrypoint(args, DefaultTerminal, &mut buffer);
        assert!(result.is_ok());
        assert_eq!(
            buffer.0,
            format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")).as_bytes()
        );
    }
}
