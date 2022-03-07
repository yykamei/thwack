use std::ffi::OsString;

use crate::error::{Error, Result};
use crate::preferences::Preferences;
use crate::status_line::StatusLine;

// TODO: --no-exec? might be required; users sometimes want to execute the file itself.
pub const HELP: &str = "thwack
Find a file and open it with an arbitrary command.

USAGE:
    thwack [OPTIONS] [--] [query]

ARGS:
    <query>                   The name of the file you'd like to find

OPTIONS:
    --exec <COMMAND>          Change the execution command from the default.
                              This is run when you hit the Enter on a path.
                              The default command is \"notepad\" on Windows, or \"cat\" on other platforms.
    --log-file <PATH>         Log what the program is doing to the specified PATH.
                              Log information is not output by default.
    --starting-point <PATH>   Change the starting point from the default (\".\").
    --status-line <TYPE>      Change the information on the status line.
                              The possible values are \"absolute\", \"relative\", and \"none.\"
                              The default is \"absolute.\"
    --no-gitignore            Do not respect .gitignore and search all paths including Git ignored paths.
    -h, --help                Prints help information.
    -v, --version             Prints version info and exit

ENVIRONMENT VARIABLES:
    THWACK_EXEC               The command to use when you hit the Enter key.
                              This affects the same way as \"--exec\".
                              If both are set, the value of \"--exec\" takes precedence.
    THWACK_LOG_FILE           The path to the file to log to.
                              This affects the same way as \"--log-file\".
                              If both are set, the value of \"--log-file\" takes precedence.
";

// TODO: Config might be required: e.g. impl From<Args> for Config.
pub(crate) struct Args<A: Iterator<Item = OsString>> {
    args: A,
    preferences: Preferences,
}

impl<A: Iterator<Item = OsString>> Args<A> {
    pub(crate) fn new<V: Iterator<Item = (OsString, OsString)>>(args: A, vars_os: V) -> Self {
        Self {
            args,
            preferences: Preferences::default().parse_env(vars_os),
        }
    }

    pub(crate) fn parse(mut self) -> Result<Preferences> {
        self.next()
            .expect("The first argument is supposed to be a program name")?;

        let mut query = None;

        while let Some(arg) = self.next() {
            let arg = arg?;
            match arg.as_ref() {
                "--" => self.consume_rest_as_arg()?,
                "-h" => self.set_help(true),
                "--help" => self.set_help(true),
                "-v" => self.set_version(true),
                "--version" => self.set_version(true),
                "--exec" => self.set_exec(None)?,
                "--starting-point" => self.set_starting_point(None)?,
                "--status-line" => self.set_status_line(None)?,
                "--no-gitignore" => self.preferences.gitignore = false,
                "--log-file" => self.set_log_file(None)?,
                x if x.starts_with("--exec=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_exec(Some(val))?;
                    }
                }
                x if x.starts_with("--starting-point=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_starting_point(Some(val))?;
                    }
                }
                x if x.starts_with("--status-line=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_status_line(Some(val))?;
                    }
                }
                x if x.starts_with("--log-file=") => {
                    if let Some((_, val)) = x.split_once('=') {
                        self.set_log_file(Some(val))?;
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
            self.preferences.query = q;
        }

        Ok(self.preferences)
    }

    fn next(&mut self) -> Option<Result<String>> {
        let arg = self.args.next()?;
        let error = Error::args(&format!(
            "{}\n\nThe specified argument {:?} does not seem to be valid unicode.",
            HELP, arg,
        ));
        Some(arg.to_str().map(|s| s.to_string()).ok_or(error))
    }

    fn consume_rest_as_arg(&mut self) -> Result<()> {
        let mut rest: Vec<String> = Vec::with_capacity(16);
        while let Some(val) = self.next() {
            let val = val?;
            rest.push(val);
        }
        self.preferences.query = rest.join(" ");
        Ok(())
    }

    fn set_help(&mut self, value: bool) {
        self.preferences.help = value;
    }

    fn set_version(&mut self, value: bool) {
        self.preferences.version = value;
    }

    fn set_starting_point(&mut self, value: Option<&str>) -> Result<()> {
        self.preferences.starting_point = self.arg_value("--starting-point", value)?;
        Ok(())
    }

    fn set_status_line(&mut self, value: Option<&str>) -> Result<()> {
        let value = self.arg_value("--status-line", value)?;
        self.preferences.status_line = StatusLine::try_from(value).map_err(|(_, given)| Error::args(&format!("The argument of \"--status-line\" must be one of \"absolute\", \"relative\", or \"none\": {:?} was given.", given)))?;
        Ok(())
    }

    fn set_log_file(&mut self, value: Option<&str>) -> Result<()> {
        self.preferences.log_file = Some(self.arg_value("--log-file", value)?);
        Ok(())
    }

    fn set_exec(&mut self, value: Option<&str>) -> Result<()> {
        self.preferences.exec = self.arg_value("--exec", value)?;
        Ok(())
    }

    fn arg_value(&mut self, option: &str, value: Option<&str>) -> Result<String> {
        let val = if let Some(val) = value {
            String::from(val)
        } else if let Some(val) = self.next() {
            let val = val?;
            if val.starts_with('-') {
                return Err(Error::args(&format!(
                    "{}\n\n\"{}\" needs a value.",
                    HELP, option
                )));
            }
            val
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
            Preferences::default()
        };
    }

    #[test]
    fn parser_with_double_hyphen() {
        assert_eq!(
            Args::new(
                args!["program", "-h", "--", "abc", "def", "ok!"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                help: true,
                query: String::from("abc def ok!"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--starting-point=/tmp", "--", "hey"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                starting_point: String::from("/tmp"),
                query: String::from("hey"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--", "--abc", "--version", "-h=ok!"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                query: String::from("--abc --version -h=ok!"),
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_help() {
        assert_eq!(
            Args::new(args!["program", "-h"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                help: true,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--help"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                help: true,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "-h", "--help"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                help: true,
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_version() {
        assert_eq!(
            Args::new(args!["program", "-v"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                version: true,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "-h", "--version"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                help: true,
                version: true,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "-v", "--help"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                help: true,
                version: true,
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_starting_point() {
        assert_eq!(
            Args::new(args!["program", "--starting-point=./abc"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                starting_point: String::from("./abc"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--starting-point", "./abc"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                starting_point: String::from("./abc"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--starting-point==ok="], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                starting_point: String::from("=ok="),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--starting-point", "xyz", "--help", "query"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                help: true,
                query: String::from("query"),
                starting_point: String::from("xyz"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--starting-point"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--starting-point", "--"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--starting-point="], [].into_iter())
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
    fn parser_with_log_file() {
        assert_eq!(
            Args::new(args!["program", "--log-file=./log.txt"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                log_file: Some(String::from("./log.txt")),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--log-file", "./abc"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                log_file: Some(String::from("./abc")),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--log-file==ok="], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                log_file: Some(String::from("=ok=")),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--log-file", "xyz", "--help", "query"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                help: true,
                query: String::from("query"),
                log_file: Some(String::from("xyz")),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--log-file"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--log-file\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--log-file", "--"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--log-file\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--log-file="], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!(
                "{}\n\n\"--log-file\" needs a value. Empty string cannot be processed.",
                HELP
            ),
        );
    }

    #[test]
    fn parser_with_status_line() {
        assert_eq!(
            Args::new(args!["program", "--status-line=none"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                status_line: StatusLine::None,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--status-line=relative"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                status_line: StatusLine::Relative,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--status-line=absolute"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                status_line: StatusLine::Absolute,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--status-line", "none"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                status_line: StatusLine::None,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--status-line", "relative"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                status_line: StatusLine::Relative,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args!["program", "--status-line", "absolute"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                status_line: StatusLine::Absolute,
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--status-line=unknown"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("The argument of \"--status-line\" must be one of \"absolute\", \"relative\", or \"none\": \"unknown\" was given."),
        );
        assert_eq!(
            Args::new(args!["program", "--status-line"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--status-line\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--status-line", "--"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--status-line\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--status-line="], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!(
                "{}\n\n\"--status-line\" needs a value. Empty string cannot be processed.",
                HELP
            ),
        );
    }

    #[test]
    fn parser_with_no_gitignore() {
        assert_eq!(
            Args::new(args!["program", "--no-gitignore"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                gitignore: false,
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_starting_point_disallow_option_like_value() {
        assert_eq!(
            Args::new(
                args!["program", "--starting-point", "--option-like-value"],
                [].into_iter()
            )
            .parse()
            .unwrap_err()
            .message,
            format!("{}\n\n\"--starting-point\" needs a value.", HELP),
        );
    }

    #[test]
    fn parser_with_starting_point_allow_option_like_value() {
        assert_eq!(
            Args::new(
                args!["program", "--starting-point=--option-like-value"],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                starting_point: String::from("--option-like-value"),
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_exec() {
        assert_eq!(
            Args::new(args!["program", "--exec=bat"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                exec: String::from("bat"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--exec", "open"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                exec: String::from("open"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--exec==ok="], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                exec: String::from("=ok="),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(
                args![
                    "program",
                    "--starting-point",
                    "xyz",
                    "--exec=fire",
                    "-v",
                    "query"
                ],
                [].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                version: true,
                query: String::from("query"),
                exec: String::from("fire"),
                starting_point: String::from("xyz"),
                ..default!()
            }
        );
        assert_eq!(
            Args::new(args!["program", "--exec"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--exec\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--exec", "--"], [].into_iter())
                .parse()
                .unwrap_err()
                .message,
            format!("{}\n\n\"--exec\" needs a value.", HELP),
        );
        assert_eq!(
            Args::new(args!["program", "--exec="], [].into_iter())
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
    fn parser_with_exec_should_be_used_even_though_its_environment_variable_is_set() {
        assert_eq!(
            Args::new(
                args!["program", "--exec=bat"],
                [(OsString::from("THWACK_EXEC"), OsString::from("/bin/echo"))].into_iter()
            )
            .parse()
            .unwrap(),
            Preferences {
                exec: String::from("bat"),
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_exec_allow_option_like_value() {
        assert_eq!(
            Args::new(args!["program", "--exec=--special--"], [].into_iter())
                .parse()
                .unwrap(),
            Preferences {
                exec: String::from("--special--"),
                ..default!()
            }
        );
    }

    #[test]
    fn parser_with_exec_disallow_option_like_value() {
        assert_eq!(
            Args::new(
                args!["program", "--exec", "--option-like-value"],
                [].into_iter()
            )
            .parse()
            .unwrap_err()
            .message,
            format!("{}\n\n\"--exec\" needs a value.", HELP),
        );
    }
}
