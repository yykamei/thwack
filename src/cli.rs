use std::env::Args;
use std::process::exit;

pub fn entrypoint(args: &mut Args) {
    let name = args.next().expect("The first argument is supposed to be a program name.");
    match args.next() {
        Some(query) => println!("You hit this command with: `{} {}`", name, query),
        None => {
            eprintln!("USAGE: {} QUERY", name);
            exit(1);
        },
    }
}
