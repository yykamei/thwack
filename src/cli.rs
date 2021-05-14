use std::env::{current_dir, Args};
use std::io::Write;
use std::process::exit;

use crate::error::{Error, Result};
use crate::finder::Finder;

pub fn entrypoint(args: &mut Args) -> Result<()> {
    let name = args
        .next()
        .expect("The first argument is supposed to be a program name.");
    match args.next() {
        Some(query) => {
            let finder = Finder::new(current_dir()?, &query)?;
            for path in finder {
                let path = path?;
                println!("{}", path.to_string_lossy()); // FIXME: Implement more appropriately, plus sorting is required.
            }
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
