use std::io::Write;
use std::time::Duration;

use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use crossterm::style::Attribute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, execute, queue, style};
use git2::Repository;
use log::{debug, info, trace, warn};

use crate::candidates::Candidates;
use crate::error::Result;
use crate::invoke::{invoke, Libc};
use crate::preferences::Preferences;
use crate::query::Query;
use crate::starting_point::StartingPoint;
use crate::status_line::StatusLine;
use crate::{Error, Terminal};

macro_rules! ctrl {
    ($char:expr) => {
        KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    };
}
macro_rules! char {
    ($char:ident) => {
        KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    };
    ($char:expr) => {
        KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    };
}

macro_rules! esc {
    () => {
        KeyEvent {
            code: KeyCode::Esc,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! backspace {
    () => {
        KeyEvent {
            code: KeyCode::Backspace,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! up {
    () => {
        KeyEvent {
            code: KeyCode::Up,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! down {
    () => {
        KeyEvent {
            code: KeyCode::Down,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! left {
    () => {
        KeyEvent {
            code: KeyCode::Left,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! right {
    () => {
        KeyEvent {
            code: KeyCode::Right,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

macro_rules! enter {
    () => {
        KeyEvent {
            code: KeyCode::Enter,
            modifiers: _,
            kind: KeyEventKind::Press,
            state: _,
        }
    };
}

pub(crate) struct Screen<'a, T: Terminal, W: Write> {
    preferences: &'a Preferences,
    query: Query,
    starting_point: StartingPoint,
    repo: Option<Repository>,
    candidates: Candidates,
    clipboard: Option<ClipboardContext>,
    terminal: &'a T,
    stdout: &'a mut W,
}

impl<'a, T: Terminal, W: Write> Screen<'a, T, W> {
    pub(crate) fn new(
        preferences: &'a Preferences,
        terminal: &'a T,
        stdout: &'a mut W,
    ) -> Result<Screen<'a, T, W>> {
        let query = Query::new(&preferences.query);
        let starting_point = StartingPoint::new(&preferences.starting_point)?;
        let visible = visible_paths_length(terminal, &preferences)?;
        let repo = if preferences.gitignore {
            match Repository::discover(&preferences.starting_point) {
                Ok(r) => Some(r),
                Err(_) => {
                    info!(
                        "The starting point `{}` is not a Git repository",
                        &preferences.starting_point
                    );
                    None
                }
            }
        } else {
            None
        };
        let candidates = Candidates::new(visible, &starting_point, &query, repo.as_ref())?;
        let clipboard = match ClipboardContext::new().map_err(|e| Error::clipboard(e)) {
            Ok(c) => Some(c),
            Err(e) => {
                warn!("Failed to initialize clipboard: {}", e);
                None
            }
        };

        Ok(Screen {
            preferences,
            query,
            starting_point,
            repo,
            candidates,
            clipboard,
            terminal,
            stdout,
        })
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        execute!(self.stdout, EnterAlternateScreen, style::ResetColor)?;
        self.terminal
            .enable_raw_mode()
            .expect("Failed to enable raw mode");
        match self.render() {
            Ok(_) => {}
            Err(e) => {
                self.leave_terminal().expect("Failed to disable raw mode");
                return Err(e);
            }
        }
        match self.poll() {
            Ok(_) => {}
            Err(e) => {
                self.leave_terminal().expect("Failed to disable raw mode");
                return Err(e);
            }
        }
        self.leave_terminal()?;
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        loop {
            if !self.terminal.poll(Duration::from_millis(300))? {
                continue;
            }
            let event = ThwackEvent::from(self.terminal.read()?);
            trace!("event={:?}, query={}", &event, &self.query);
            match event {
                ThwackEvent::Quit => break,
                ThwackEvent::QueryPush(c) => {
                    self.query.push(c);
                    self.candidates = Candidates::new(
                        visible_paths_length(self.terminal, &self.preferences)?,
                        &self.starting_point,
                        &self.query,
                        self.repo.as_ref(),
                    )?;
                    self.render()?;
                }
                ThwackEvent::QueryPop => {
                    self.query.pop();
                    self.candidates = Candidates::new(
                        visible_paths_length(self.terminal, &self.preferences)?,
                        &self.starting_point,
                        &self.query,
                        self.repo.as_ref(),
                    )?;
                    self.render()?;
                }
                ThwackEvent::Up => {
                    self.candidates.move_up();
                    self.render()?;
                }
                ThwackEvent::Down => {
                    self.candidates.move_down();
                    self.render()?;
                }
                ThwackEvent::Left => {
                    self.query.move_left();
                    self.render()?;
                }
                ThwackEvent::Right => {
                    self.query.move_right();
                    self.render()?;
                }
                ThwackEvent::Invoke => {
                    let path: Option<String> = match self.candidates.selected() {
                        None => None,
                        Some(p) => match self.preferences.status_line {
                            StatusLine::None | StatusLine::Absolute => {
                                Some(p.absolute().to_string())
                            }
                            StatusLine::Relative => Some(p.relative().to_string()),
                        },
                    };
                    if let Some(p) = path {
                        self.leave_terminal()?;
                        invoke(&Libc, &self.preferences, &p)?;
                    }
                }
                ThwackEvent::CopyAbsolutePath | ThwackEvent::CopyRelativePath => {
                    if let Some(c) = self.clipboard.as_mut() {
                        if let Some(path) = self.candidates.selected() {
                            let path = match event {
                                ThwackEvent::CopyAbsolutePath => path.absolute(),
                                ThwackEvent::CopyRelativePath => path.relative(),
                                _ => unreachable!(),
                            };
                            c.set_contents(path.to_string())
                                .map_err(|e| Error::clipboard(e))?;
                        }
                        // TODO: Feedback to the user when the copy operation fails.
                    }
                    break;
                }
                ThwackEvent::TerminalResize => {
                    self.candidates = Candidates::new(
                        visible_paths_length(self.terminal, &self.preferences)?,
                        &self.starting_point,
                        &self.query,
                        self.repo.as_ref(),
                    )?;
                    self.render()?;
                }
                ThwackEvent::None => {}
            }
        }
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.clear()?;
        self.render_query()?;
        self.render_candidates()?;
        self.render_status()?;
        self.render_short_help()?;
        self.place_cursor()?;
        self.flush()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        queue!(self.stdout, Clear(ClearType::All))?;
        Ok(())
    }

    fn place_cursor(&mut self) -> Result<()> {
        let x = self.query.terminal_pos as u16 + 8; // 8 is the length of "Search: ".
        queue!(self.stdout, cursor::MoveTo(x, 0))?;
        Ok(())
    }

    fn render_query(&mut self) -> Result<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            style::Print("Search: "),
            style::Print(&self.query),
        )?;
        Ok(())
    }

    fn render_candidates(&mut self) -> Result<()> {
        let selected = self.candidates.selected();
        let (columns, _) = self.terminal.size()?;

        queue!(self.stdout, cursor::MoveTo(0, 1),)?;

        for candidate in self.candidates.paths() {
            match selected {
                Some(s) if s == candidate => {
                    queue!(self.stdout, style::Print("> "),)?;
                }
                _ => {
                    queue!(self.stdout, style::Print("  "),)?;
                }
            }
            for chunk in candidate.relative_chunks((columns - 2).into()) {
                if chunk.matched() {
                    queue!(
                        self.stdout,
                        style::SetAttribute(Attribute::Bold),
                        style::Print(format!("{}", chunk)),
                        style::SetAttribute(Attribute::Reset),
                    )?;
                } else {
                    queue!(self.stdout, style::Print(format!("{}", chunk)))?;
                }
            }
            queue!(self.stdout, cursor::MoveToNextLine(1))?;
        }
        Ok(())
    }

    fn render_status(&mut self) -> Result<()> {
        let selected = self.candidates.selected();
        let (columns, rows) = self.terminal.size()?;

        let message = match (selected, self.preferences.status_line) {
            (_, StatusLine::None) => None,
            (None, _) => Some("No matching files found.".to_string()),
            (Some(s), StatusLine::Absolute) => Some(s.truncated_absolute(columns as usize)),
            (Some(s), StatusLine::Relative) => Some(s.truncated_relative(columns as usize)),
        };
        if let Some(ref m) = message {
            queue!(
                self.stdout,
                cursor::MoveTo(0, rows - 2),
                style::SetAttribute(Attribute::Bold),
                style::SetAttribute(Attribute::Reverse),
                style::Print(format!("{:width$}", m, width = columns as usize)),
                style::SetAttribute(Attribute::Reset),
            )?;
        }
        Ok(())
    }

    fn render_short_help(&mut self) -> Result<()> {
        let (columns, rows) = self.terminal.size()?;

        // TODO: This number is the same amount of columns occupied by this short help.
        if columns < 97 {
            info!("Terminal is too small to render short help, {}", columns);
            return Ok(());
        }
        queue!(
            self.stdout,
            cursor::MoveTo(0, rows - 1),
            style::SetAttribute(Attribute::Bold),
            style::Print("<Up>/<Ctrl-p>:"),
            style::SetAttribute(Attribute::Reset),
            cursor::MoveRight(1),
            style::Print("Up"),
            cursor::MoveRight(2),
            style::SetAttribute(Attribute::Bold),
            style::Print("<Down>/<Ctrl-n>:"),
            style::SetAttribute(Attribute::Reset),
            cursor::MoveRight(1),
            style::Print("Down"),
            cursor::MoveRight(2),
            style::SetAttribute(Attribute::Bold),
            style::Print("<Enter>:"),
            style::SetAttribute(Attribute::Reset),
            cursor::MoveRight(1),
            style::Print("Execute"),
            cursor::MoveRight(2),
            style::SetAttribute(Attribute::Bold),
            style::Print("<C-d>/<C-y>:"),
            style::SetAttribute(Attribute::Reset),
            cursor::MoveRight(1),
            style::Print("Copy (relative/absolute)"),
        )?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.stdout.flush()?;
        Ok(())
    }

    fn leave_terminal(&mut self) -> Result<()> {
        execute!(self.stdout, LeaveAlternateScreen)?;
        self.terminal.disable_raw_mode()?;
        debug!("Terminal left");
        Ok(())
    }
}

fn visible_paths_length(terminal: &dyn Terminal, preferences: &Preferences) -> Result<usize> {
    let (_, rows) = terminal.size()?;
    let mut visible = match preferences.status_line {
        StatusLine::None => rows - 2,
        _ => rows - 3,
    };
    visible = if visible < 1 { 0 } else { visible };
    Ok(visible as usize)
}

#[derive(Debug)]
enum ThwackEvent {
    Quit,
    QueryPush(char),
    QueryPop,
    Up,
    Down,
    Left,
    Right,
    Invoke,
    CopyAbsolutePath,
    CopyRelativePath,
    TerminalResize,
    None,
}

impl From<Event> for ThwackEvent {
    fn from(ev: Event) -> ThwackEvent {
        match ev {
            Event::Key(event) => match event {
                ctrl!('c') | esc!() => ThwackEvent::Quit,
                char!(c) => ThwackEvent::QueryPush(c),
                backspace!() => ThwackEvent::QueryPop,
                up!() | ctrl!('p') => ThwackEvent::Up,
                down!() | ctrl!('n') => ThwackEvent::Down,
                left!() => ThwackEvent::Left,
                right!() => ThwackEvent::Right,
                enter!() => ThwackEvent::Invoke,
                ctrl!('y') => ThwackEvent::CopyAbsolutePath,
                ctrl!('d') => ThwackEvent::CopyRelativePath,
                _ => ThwackEvent::None,
            },
            Event::Resize(_, _) => ThwackEvent::TerminalResize,
            _ => ThwackEvent::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ascii::escape_default;
    use std::collections::VecDeque;
    use std::fmt::{Debug, Formatter};
    use std::io;
    use std::str::from_utf8;
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Debug, Default)]
    struct MockTerminal {
        events: Arc<Mutex<VecDeque<Event>>>,
        size: (u16, u16),
    }

    impl MockTerminal {
        fn size(mut self, columns: u16, rows: u16) -> Self {
            self.size = (columns, rows);
            self
        }

        fn add_event(self, event: Event) -> Self {
            self.events.lock().unwrap().push_back(event);
            self
        }
    }

    impl Terminal for MockTerminal {
        fn size(&self) -> Result<(u16, u16)> {
            Ok(self.size)
        }
        fn enable_raw_mode(&self) -> Result<()> {
            Ok(())
        }
        fn disable_raw_mode(&self) -> Result<()> {
            Ok(())
        }
        fn poll(&self, _timeout: Duration) -> Result<bool> {
            let data = self.events.clone();
            let events = data.lock().unwrap();
            let event = events.front();
            Ok(event.is_some())
        }
        fn read(&self) -> Result<Event> {
            let e = self.events.clone().lock().unwrap().pop_front().unwrap();
            Ok(e)
        }
    }

    #[derive(Eq, PartialEq)]
    pub struct Buffer {
        inner: Vec<u8>,
    }

    impl Buffer {
        pub fn new() -> Self {
            Self { inner: vec![] }
        }
    }

    impl Debug for Buffer {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let mut buf: Vec<u8> = Vec::with_capacity(self.inner.len());
            let _ = &self.inner.iter().for_each(|v| {
                for e in escape_default(*v) {
                    buf.push(e);
                }
            });
            let inner = from_utf8(&buf).unwrap();
            Debug::fmt(inner, f)
        }
    }

    impl Write for Buffer {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let _ = self.inner.extend(buf);
            Ok(self.inner.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            let _ = self.inner.extend(buf);
            Ok(())
        }
    }

    #[test]
    fn test_screen() {
        let terminal = MockTerminal::default()
            .size(98, 20)
            .add_event(Event::Key(char!('a')))
            .add_event(Event::Key(char!('b')))
            .add_event(Event::Key(char!('c')))
            .add_event(Event::Key(char!('😇')))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Left.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Right.into()))
            .add_event(Event::Key(KeyCode::Backspace.into()))
            .add_event(Event::Key(KeyCode::Backspace.into()))
            .add_event(Event::Key(KeyCode::Backspace.into()))
            .add_event(Event::Key(KeyCode::Up.into()))
            .add_event(Event::Key(KeyCode::Up.into()))
            .add_event(Event::Key(KeyCode::Down.into()))
            .add_event(Event::Key(KeyCode::Down.into()))
            .add_event(Event::Resize(100, 30))
            .add_event(Event::Key(KeyCode::Esc.into()));
        let preferences = Preferences::default();
        let mut buffer = Buffer::new();
        let mut screen = Screen::new(&preferences, &terminal, &mut buffer).unwrap();
        screen.start().unwrap();
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_clipboard() {
        let terminal = MockTerminal::default()
            .size(98, 20)
            .add_event(Event::Key(ctrl!('d')))
            .add_event(Event::Key(KeyCode::Esc.into()));
        let mut preferences = Preferences::default();
        preferences.query = "README.md".to_string();
        let mut buffer = Buffer::new();
        let mut screen = Screen::new(&preferences, &terminal, &mut buffer).unwrap();
        screen.start().unwrap();

        let mut ctx = ClipboardContext::new().unwrap();
        assert_eq!(ctx.get_contents().unwrap(), "README.md");
    }
}
