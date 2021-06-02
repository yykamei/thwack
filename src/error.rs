use std::error;
use std::ffi::NulError;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::result::Result as StdResult;

const FAILURE: i32 = 1;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    Args,
    InsufficientQuery, // TODO: Delete
    InvalidUnicode,
    IO,
    Terminal,
    Exec,
    NulError,
}

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
    pub kind: ErrorKind,
    pub source: Option<Box<dyn error::Error>>,
    pub exit_code: i32,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.message, f)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.source.as_deref()
    }
}

impl Error {
    pub fn args(message: &str) -> Self {
        // TODO: Maybe, more information is required?
        Self {
            message: message.to_string(),
            kind: ErrorKind::Args,
            source: None,
            exit_code: FAILURE,
        }
    }

    pub fn insufficient_query(message: &str) -> Self {
        Self {
            message: message.to_string(),
            kind: ErrorKind::InsufficientQuery,
            source: None,
            exit_code: FAILURE,
        }
    }

    pub fn invalid_unicode(message: &str) -> Self {
        Self {
            message: message.to_string(),
            kind: ErrorKind::InvalidUnicode,
            source: None,
            exit_code: FAILURE,
        }
    }

    pub fn exec(message: &str) -> Self {
        Self {
            message: message.to_string(),
            kind: ErrorKind::Exec,
            source: None,
            exit_code: FAILURE,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            message: format!(
                "Unhandled IO error happened. See the details from .source: {}",
                error
            ),
            kind: ErrorKind::IO,
            source: Some(Box::new(error)),
            exit_code: FAILURE,
        }
    }
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self {
            message: format!(
                "Unhandled terminal error happened. See the details from .source: {}",
                error
            ),
            kind: ErrorKind::Terminal,
            source: Some(Box::new(error)),
            exit_code: FAILURE,
        }
    }
}

impl From<NulError> for Error {
    fn from(error: NulError) -> Self {
        Self {
            message: format!(
                "The string contains nul bytes. See the details from .source: {}",
                error
            ),
            kind: ErrorKind::NulError,
            source: Some(Box::new(error)),
            exit_code: FAILURE,
        }
    }
}
