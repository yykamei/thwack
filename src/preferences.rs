use std::ffi::OsString;

use crate::status_line::StatusLine;

#[derive(Debug, PartialEq)]
pub(crate) struct Preferences {
    pub(crate) help: bool,
    pub(crate) version: bool,
    pub(crate) gitignore: bool,
    pub(crate) starting_point: String,
    pub(crate) status_line: StatusLine,
    pub(crate) log_file: Option<String>,
    pub(crate) query: String,
    pub(crate) exec: String,
}

impl Preferences {
    pub(crate) fn parse_env<V: Iterator<Item = (OsString, OsString)>>(
        mut self,
        vars_os: V,
    ) -> Self {
        for (key, value) in vars_os {
            match (key.to_str(), value.to_str()) {
                (Some("THWACK_LOG_FILE"), Some(value)) => {
                    log::info!("Set log_file to {} from THWACK_LOG_FILE", value);
                    self.log_file = Some(value.to_string())
                }
                (Some("THWACK_EXEC"), Some(value)) => {
                    log::info!("Set exec to {} from THWACK_EXEC", value);
                    self.exec = value.to_string()
                }
                _ => {
                    log::debug!("Ignoring env var: {:?}", key);
                    continue;
                }
            }
        }
        self
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            help: false,
            version: false,
            gitignore: true,
            starting_point: String::from("."),
            status_line: StatusLine::Absolute,
            log_file: None,
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

    #[test]
    fn parse_env_without_thwack_environment_variables() {
        let preferences = Preferences::default().parse_env([].into_iter());
        assert_eq!(preferences, Preferences::default());
    }

    #[test]
    fn parse_env_with_dummy_envs() {
        let preferences = Preferences::default().parse_env(
            [
                (
                    OsString::from("THWACK_LOG_FILE"),
                    OsString::from("/tmp/thwack.log"),
                ),
                (OsString::from("THWACK_EXEC"), OsString::from("/bin/echo")),
            ]
            .into_iter(),
        );
        assert_eq!(
            preferences,
            Preferences {
                log_file: Some(String::from("/tmp/thwack.log")),
                exec: String::from("/bin/echo"),
                ..Preferences::default()
            }
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
            Preferences::default(),
            Preferences {
                help: false,
                version: false,
                gitignore: true,
                starting_point: String::from("."),
                status_line: StatusLine::Absolute,
                log_file: None,
                query: String::from(""),
                exec,
            }
        );
    }
}
