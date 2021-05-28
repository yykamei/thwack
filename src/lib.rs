pub use cli::{entrypoint, safe_exit};
pub use error::{Error, ErrorKind, Result};

mod args;
mod cli;
mod error;
mod finder;
mod matched_path;
