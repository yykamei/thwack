pub use cli::{entrypoint, safe_exit};
pub use error::{Error, ErrorKind, Result};
pub use preferences::HELP;
pub use terminal::{DefaultTerminal, DefaultTerminalEvent, Terminal, TerminalEvent};

mod cli;
mod error;
mod finder;
mod logger;
mod matched_path;
mod preferences;
mod starting_point;
mod terminal;
