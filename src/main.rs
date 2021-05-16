use std::env;

use pinpoint::{entrypoint, safe_exit};
use std::io::stdout;

fn main() {
    match entrypoint(env::args_os(), stdout()) {
        Ok(_) => safe_exit(0),
        Err(e) => {
            eprintln!("{}", e); // TODO: Write a more readable error message.
            safe_exit(e.exit_code);
        }
    }
}
