use std::ffi::OsString;

use thwack::entrypoint;

mod helper;

fn version() -> String {
    format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

#[test]
fn show_version() {
    let args = args!["thwack", "--version"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(version()));
}

#[test]
fn show_version_with_query() {
    let args = args!["thwack", "--version", "--", "query"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(version()));
}

#[test]
fn show_version_with_starting_point() {
    let args = args!["thwack", "--version", "--starting-point=/tmp"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(version()));
}
