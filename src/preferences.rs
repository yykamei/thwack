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
