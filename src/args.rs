use std::ffi::OsString;

use crate::error::{Error, Result};

// TODO: --no-exec? might be required; users sometimes want to execute the file itself.
pub(crate) const HELP: &str = "thwack
Find a file and open it with an arbitrary command.

USAGE:
    thwack [OPTIONS] [query]

ARGS:
    <query>                   The name of the file you'd like to find

OPTIONS:
    --exec <COMMAND>          Change the execution command from the default.
                              This is run when you hit the Enter on a path
                              The default command is \"notepad\" on Windows, or \"cat\" on other platforms.
    --starting-point <PATH>   Change the starting point from the default (\".\")
    -h, --help                Prints help information
    -v, --version             Prints version info and exit
";

// TODO: Config might be required: e.g. impl From<Args> for Config.
pub(crate) struct Parser<A: Iterator<Item = OsString>> {
    args: A,
    parsed_args: ParsedArgs,
}

impl<A: Iterator<Item = OsString>> Parser<A> {
    pub(crate) fn new(args: A) -> Self {
        Self {
            args,
            parsed_args: ParsedArgs::default(),
        }
    }

    pub(crate) fn parse(mut self) -> Result<ParsedArgs> {
        self.next()
            .expect("The first argument is supposed to be a program name")?;

        let mut query = None;

        while let Some(arg) = self.next() {
            let arg = arg?;
            match arg.as_ref() {
                "-h" => self.set_help(true),
                "--help" => self.set_help(true),
                "-v" => self.set_version(true),
                "--version" => self.set_version(true),
                "--starting-point" => self.set_starting_point(None)?,
                "--exec" => self.set_exec(None)?,
                x if x.starts_with("--starting-point=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_starting_point(Some(val))?;
                    }
                }
                x if x.starts_with("--exec=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_exec(Some(val))?;
                    }
                }
                x => {
                    if query.is_none() {
                        query = Some(x.to_string());
                    } else {
                        return Err(Error::args(&format!(
                            "{}\n\nIllegal argument: {:?}",
                            HELP, x
                        )));
                    }
                }
            }
        }
        if let Some(q) = query {
            self.parsed_args.query = q;
        }

        Ok(self.parsed_args)
    }

    fn next(&mut self) -> Option<Result<String>> {
        let arg = self.args.next()?;
        let error = Error::args(&format!(
            "{}\n\nThe specified argument {:?} does not seem to be valid unicode.",
            HELP, arg,
        ));
        Some(arg.to_str().map(|s| s.to_string()).ok_or(error))
    }

    fn set_help(&mut self, value: bool) {
        self.parsed_args.help = value;
    }

    fn set_version(&mut self, value: bool) {
        self.parsed_args.version = value;
    }

    fn set_starting_point(&mut self, value: Option<&str>) -> Result<()> {
        self.parsed_args.starting_point = self.arg_value("--starting-point", value)?;
        Ok(())
    }

    fn set_exec(&mut self, value: Option<&str>) -> Result<()> {
        self.parsed_args.exec = self.arg_value("--exec", value)?;
        Ok(())
    }

    fn arg_value(&mut self, option: &str, value: Option<&str>) -> Result<String> {
        let val = if let Some(val) = value {
            String::from(val)
        } else if let Some(val) = self.next() {
            val?
        } else {
            return Err(Error::args(&format!(
                "{}\n\n\"{}\" needs a value.",
                HELP, option
            )));
        };
        if val.is_empty() {
            return Err(Error::args(&format!(
                "{}\n\n\"{}\" needs a value. Empty string cannot be processed.",
                HELP, option
            )));
        }
        Ok(val)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct ParsedArgs {
    pub(crate) help: bool,
    pub(crate) version: bool,
    pub(crate) starting_point: String,
    pub(crate) query: String,
    pub(crate) exec: String,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            help: false,
            version: false,
            starting_point: String::from("."),
            query: String::from(""),
            exec: if cfg!(windows) {
                String::from("notepad")
            } else {
                String::from("cat")
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! args {
        ($($x:expr),+ $(,)?) => {
            vec![$(OsString::from($x)),+].into_iter()
        };
    }

    macro_rules! default {
        () => {
            ParsedArgs::default()
        };
    }

    #[test]
    fn parser_with_help() {
        assert_eq!(
            Parser::new(args!["program", "-h"]).parse().unwrap(),
            ParsedArgs {
                help: true,
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--help"]).parse().unwrap(),
            ParsedArgs {
                help: true,
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "-h", "--help"])
                .parse()
                .unwrap(),
            ParsedArgs {
                help: true,
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_version() {
        assert_eq!(
            Parser::new(args!["program", "-v"]).parse().unwrap(),
            ParsedArgs {
                version: true,
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "-h", "--version"])
                .parse()
                .unwrap(),
            ParsedArgs {
                help: true,
                version: true,
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "-v", "--help"])
                .parse()
                .unwrap(),
            ParsedArgs {
                help: true,
                version: true,
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_starting_point() {
        assert_eq!(
            Parser::new(args!["program", "--starting-point=./abc"])
                .parse()
                .unwrap(),
            ParsedArgs {
                starting_point: String::from("./abc"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--starting-point", "./abc"])
                .parse()
                .unwrap(),
            ParsedArgs {
                starting_point: String::from("./abc"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--starting-point=--option-like-value"])
                .parse()
                .unwrap(),
            ParsedArgs {
                starting_point: String::from("--option-like-value"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--starting-point==ok="])
                .parse()
                .unwrap(),
            ParsedArgs {
                starting_point: String::from("=ok="),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args![
                "program",
                "--starting-point",
                "xyz",
                "--help",
                "query"
            ])
            .parse()
            .unwrap(),
            ParsedArgs {
                help: true,
                query: String::from("query"),
                starting_point: String::from("xyz"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--starting-point"])
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        );
        // TODO: Enable this assertion later
        // assert_eq!(
        //     Parser::new(args!["program", "--starting-point", "--option-like-value"])
        //         .parse()
        //         .unwrap_err().message,
        //     format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        //);
        // TODO: Enable this assertion later
        // assert_eq!(
        //     Parser::new(args!["program", "--starting-point", "--df"])
        //         .parse()
        //         .unwrap_err()
        //         .message,
        //     format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        // );
        assert_eq!(
            Parser::new(args!["program", "--starting-point="])
                .parse()
                .unwrap_err()
                .message,
            format!(
                "{}\n\n\"--starting-point\" needs a value. Empty string cannot be processed.",
                HELP
            ),
        );
    }

    #[test]
    fn parser_with_exec() {
        assert_eq!(
            Parser::new(args!["program", "--exec=bat"]).parse().unwrap(),
            ParsedArgs {
                exec: String::from("bat"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--exec", "open"])
                .parse()
                .unwrap(),
            ParsedArgs {
                exec: String::from("open"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--exec=--special--"])
                .parse()
                .unwrap(),
            ParsedArgs {
                exec: String::from("--special--"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--exec==ok="])
                .parse()
                .unwrap(),
            ParsedArgs {
                exec: String::from("=ok="),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args![
                "program",
                "--starting-point",
                "xyz",
                "--exec=fire",
                "-v",
                "query"
            ])
            .parse()
            .unwrap(),
            ParsedArgs {
                version: true,
                query: String::from("query"),
                exec: String::from("fire"),
                starting_point: String::from("xyz"),
                ..default!()
            }
        );
        assert_eq!(
            Parser::new(args!["program", "--exec"])
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--exec\" needs a value.", HELP),
        );
        // TODO: Enable this assertion later
        // assert_eq!(
        //     Parser::new(args!["program", "--exec", "--option-like-value"])
        //         .parse()
        //         .unwrap_err().message,
        //     format!("{}\n\n\"--exec\" needs a value.", HELP),
        //);
        // TODO: Enable this assertion later
        // assert_eq!(
        //     Parser::new(args!["program", "--exec", "--df"])
        //         .parse()
        //         .unwrap_err()
        //         .message,
        //     format!("{}\n\n\"--exec\" needs a value.", HELP),
        // );
        assert_eq!(
            Parser::new(args!["program", "--exec="])
                .parse()
                .unwrap_err()
                .message,
            format!(
                "{}\n\n\"--exec\" needs a value. Empty string cannot be processed.",
                HELP
            ),
        );
    }

    #[test]
    fn parsed_args_returns_default() {
        let exec = if cfg!(windows) {
            String::from("notepad")
        } else {
            String::from("cat")
        };
        assert_eq!(
            ParsedArgs::default(),
            ParsedArgs {
                help: false,
                version: false,
                starting_point: String::from("."),
                query: String::from(""),
                exec,
            }
        );
    }
}
