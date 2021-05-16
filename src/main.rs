use std::env;

use pinpoint::{entrypoint, safe_exit};

fn main() {
    match entrypoint(&mut env::args_os()) {
        Ok(_) => safe_exit(0),
        Err(e) => {
            eprintln!("{}", e); // TODO: Write a more readable error message.
            safe_exit(e.exit_code);
        }
    }
}
