use pinpoint::entrypoint;
use std::env;

fn main() {
    entrypoint(&mut env::args());
}
