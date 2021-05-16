use std::env::ArgsOs;
use std::io::{self, Write};
use std::process::exit;

use crate::args::{Parser, HELP};
use crate::error::Result;
use crate::finder::Finder;

pub fn entrypoint(args: ArgsOs, mut stdout: impl Write) -> Result<()> {
    let args = Parser::new(args).parse()?;
    if args.help {
        print_help(&mut stdout)?;
        return Ok(());
    }
    for path in Finder::new(&args.starting_point, &args.query)? {
        let path = path?;
        println!("{}", path); // FIXME: Implement more appropriately, plus sorting is required.
    }
    Ok(())
}

pub fn safe_exit(code: i32) {
    let _ = std::io::stdout().lock().flush();
    let _ = std::io::stderr().lock().flush();
    exit(code)
}

fn print_help(buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_all(format!("{}\n", HELP).as_bytes())
}
