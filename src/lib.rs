pub use cli::entrypoint;
pub use cli::safe_exit;
pub use error::{Error, ErrorKind, Result};
pub use finder::{Finder, MatchedPath};

mod args;
mod cli;
mod error;
mod finder;
