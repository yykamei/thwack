use std::env::ArgsOs;
use std::path::PathBuf;

use crate::error::{Error, Result};

pub(crate) const HELP: &str = "pinpoint
Find a file and open it with an arbitrary command.

USAGE:
    pinpoint [OPTIONS] [query]

ARGS:
    <query>                   The name of the file you'd like to find

OPTIONS:
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
                x if x.starts_with("--starting-point=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_starting_point(Some(val))?;
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

        self.canonicalize_starting_point()?;
        Ok(self.parsed_args)
    }

    #[inline]
    fn next(&mut self) -> Option<Result<String>> {
        let arg = self.args.next()?;
        let error = Error::args(&format!(
            "{}\n\nThe specified string {:?} does not seem to be valid unicode.",
            HELP, arg,
        ));
        Some(arg.to_str().map(|s| s.to_string()).ok_or(error))
    }

    #[inline]
    fn set_help(&mut self, value: bool) {
        self.parsed_args.help = value;
    }

    #[inline]
    fn set_starting_point(&mut self, value: Option<&str>) -> Result<()> {
        if let Some(val) = value {
            self.parsed_args.starting_point = PathBuf::from(val);
        } else if let Some(val) = self.next() {
            self.parsed_args.starting_point = PathBuf::from(val?);
        } else {
            return Err(Error::args(&format!(
                "{}\n\n\"--starting-point\" needs a value",
                HELP
            )));
        }
        Ok(())
    }

    #[inline]
    fn canonicalize_starting_point(&mut self) -> Result<()> {
        self.parsed_args.starting_point = self.parsed_args.starting_point.canonicalize().map_err(|_e|
            Error::args(&format!(
                "{}\n\nThe specified starting point {:?} cannot be normalized. Perhaps, {:?} might not exist.",
                HELP, self.parsed_args.starting_point, self.parsed_args.starting_point,
            ))
        )?;
        Ok(())
    }
}

pub(crate) struct ParsedArgs {
    pub(crate) help: bool,
    pub(crate) starting_point: PathBuf,
    pub(crate) query: String,
}

impl Default for ParsedArgs {
    fn default() -> Self {
        Self {
            help: false,
            starting_point: PathBuf::from("."),
            query: String::from(""),
        }
    }
}
