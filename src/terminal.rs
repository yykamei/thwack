use crossterm::terminal;

use crate::error::Result;

/// Terminal is a wrapper of crossterm::terminal.
/// This is intended for mocking terminal-specific functions.
pub trait Terminal {
    fn size(&self) -> Result<(u16, u16)> {
        let pair = terminal::size()?;
        Ok(pair)
    }

    fn enable_raw_mode(&self) -> Result<()> {
        let _ = terminal::enable_raw_mode()?;
        Ok(())
    }

    fn disable_raw_mode(&self) -> Result<()> {
        let _ = terminal::disable_raw_mode()?;
        Ok(())
    }
}

pub struct DefaultTerminal;

impl Terminal for DefaultTerminal {}
