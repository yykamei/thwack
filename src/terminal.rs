use std::time::Duration;

use crossterm::event::{self, Event};
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

/// TerminalEvent is a wrapper of crossterm::event.
/// This is intended for mocking terminal-specific functions.
pub trait TerminalEvent {
    fn poll(&self, timeout: Duration) -> Result<bool> {
        let b = event::poll(timeout)?;
        Ok(b)
    }

    fn read(&mut self) -> Result<Event> {
        let e = event::read()?;
        Ok(e)
    }
}

pub struct DefaultTerminal;

pub struct DefaultTerminalEvent;

impl Terminal for DefaultTerminal {}

impl TerminalEvent for DefaultTerminalEvent {}
