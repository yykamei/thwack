pub use cli::entrypoint;
pub use cli::safe_exit;
pub use error::{Error, ErrorKind, Result};
pub use finder::Finder;
pub use matched_path::MatchedPath;

mod args;
mod cli;
mod error;
mod finder;
mod matched_path;
