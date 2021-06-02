use std::env::ArgsOs;

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
    -h, --help                Prints help information";

// TODO: Config might be required: e.g. impl From<Args> for Config.
// TODO: Test for Parser and ParsedArgs
pub(crate) struct Parser {
    args: ArgsOs,
    parsed_args: ParsedArgs,
}

impl Parser {
    pub(crate) fn new(args: ArgsOs) -> Self {
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

    // TODO: set_* could be written as the same way. Maybe, macros are useful.

    fn set_starting_point(&mut self, value: Option<&str>) -> Result<()> {
        if let Some(val) = value {
            self.parsed_args.starting_point = val.to_string();
        } else if let Some(val) = self.next() {
            self.parsed_args.starting_point = val?;
        } else {
            return Err(Error::args(&format!(
                "{}\n\n\"--starting-point\" needs a value",
                HELP
            )));
        }
        Ok(())
    }

    fn set_exec(&mut self, value: Option<&str>) -> Result<()> {
        if let Some(val) = value {
            self.parsed_args.exec = val.to_string();
        } else if let Some(val) = self.next() {
            self.parsed_args.exec = val?;
        } else {
            return Err(Error::args(&format!(
                "{}\n\n\"--exec\" needs a value",
                HELP
            )));
        }
        if self.parsed_args.exec.is_empty() {
            return Err(Error::args(&format!(
                "{}\n\n\"--exec\" needs a value. Empty string cannot be processed.",
                HELP
            )));
        }
        Ok(())
    }
}

pub(crate) struct ParsedArgs {
    pub(crate) help: bool,
    pub(crate) starting_point: String,
    pub(crate) query: String,
    pub(crate) exec: String,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            help: false,
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
