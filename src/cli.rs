use std::env::Args;
use std::io::Write;
use std::process::exit;

use crate::error::{Error, Result};

pub fn entrypoint(args: &mut Args) -> Result<()> {
    let name = args
        .next()
        .expect("The first argument is supposed to be a program name.");
    match args.next() {
        Some(query) => {
            println!("You hit this command with: `{} {}`", name, query);
            Ok(())
        }
        None => {
            eprintln!("USAGE: {} QUERY", name);
            Err(Error::insufficient_query(
                "You must pass query as the first argument.",
            ))
        }
    }
}

pub fn safe_exit(code: i32) {
    let _ = std::io::stdout().lock().flush();
    let _ = std::io::stderr().lock().flush();
    exit(code)
}
