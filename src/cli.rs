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
    stdout: &mut W,
    terminal: T,
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
