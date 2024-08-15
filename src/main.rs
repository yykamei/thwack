use std::env;
use std::io::{stderr, stdout};

use thwack::{entrypoint, safe_exit, DefaultTerminal};

fn main() {
    let mut out = stdout();
    let err = stderr();
    match entrypoint(env::args_os(), DefaultTerminal, &mut out) {
        Ok(_) => safe_exit(0, out, err),
        Err(e) => {
            eprintln!("{}", e); // TODO: Write a more readable error message.
            safe_exit(e.exit_code, out, err);
        }
    }
}
