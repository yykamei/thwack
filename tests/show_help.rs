use std::ffi::OsString;

use pretty_assertions::assert_eq;

use helper::MockTerminal;
use thwack::{entrypoint, HELP};

mod helper;

#[test]
fn show_help() {
    let args = args!["thwack", "--help"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal::new());
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(HELP));
}

#[test]
fn show_help_with_version() {
    let args = args!["thwack", "--help", "--version"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal::new());
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(HELP));
}

#[test]
fn show_help_with_query() {
    let args = args!["thwack", "--help", "--", "query"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal::new());
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(HELP));
}

#[test]
fn show_help_with_starting_point() {
    let args = args!["thwack", "--help", "--starting-point=/tmp"];
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal::new());
    assert!(result.is_ok());
    assert_eq!(buffer, buf!(HELP));
}
