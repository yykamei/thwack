use std::error;
use std::fmt::{self, Display, Formatter};
use std::result::Result as StdResult;

const FAILURE: i32 = 1;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    InsufficientQuery,
}

#[derive(Debug)]
pub struct Error {
    pub(crate) message: String,
    pub kind: ErrorKind,
    pub(crate) source: Option<Box<dyn error::Error>>,
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
    pub fn insufficient_query(message: &str) -> Self {
        Self {
            message: message.to_string(),
            kind: ErrorKind::InsufficientQuery,
            source: None,
            exit_code: FAILURE,
        }
    }
}
