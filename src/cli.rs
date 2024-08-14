use std::ffi::{CString, OsString};
use std::io::{self, Stderr, Stdout, Write};
use std::os::raw::c_char;
use std::process::exit;
use std::time::Duration;
use std::{env, ptr};

use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyEventKind, KeyEventState};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Attribute},
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use git2::Repository;

use crate::args::{Args, HELP};
use crate::error::Result;
use crate::finder::Finder;
use crate::invoke::{invoke, Libc};
use crate::matched_path::MatchedPath;
use crate::preferences::Preferences;
use crate::query::Query;
use crate::starting_point::StartingPoint;
use crate::status_line::StatusLine;
use crate::terminal::Terminal;
use crate::{logger, Error};

macro_rules! ctrl {
    ($char:expr) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($char),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    };
}

macro_rules! esc {
    () => {
        Event::Key(KeyCode::Esc.into())
    };
}

macro_rules! backspace {
    () => {
        Event::Key(KeyCode::Backspace.into())
    };
}

macro_rules! up {
    () => {
        Event::Key(KeyCode::Up.into())
    };
}

macro_rules! down {
    () => {
        Event::Key(KeyCode::Down.into())
    };
}

macro_rules! enter {
    () => {
        Event::Key(KeyCode::Enter.into())
    };
}

macro_rules! char_extracted {
    ($var:ident) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($var),
            modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    };
}

pub fn safe_exit(code: i32, stdout: Stdout, stderr: Stderr) {
    let _ = stdout.lock().flush();
    let _ = stderr.lock().flush();
    exit(code)
}

pub fn entrypoint<A: Iterator<Item = OsString>, W: Write, T: Terminal>(
    args: A,
    stdout: &mut W,
    terminal: T,
) -> Result<()> {
    let preferences = Args::new(args, env::vars_os()).parse()?;

    if let Some(ref path) = preferences.log_file {
        logger::init(path)?;
        log::debug!("Logger initialized!");
    }
    if preferences.help {
        print_and_flush(stdout, HELP)?;
        log::debug!("Show help and exit");
        return Ok(());
    }
    if preferences.version {
        print_and_flush(
            stdout,
            &format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )?;
        log::debug!("Show version and exit");
        return Ok(());
    }

    let mut runner = Runner::new(&preferences, stdout, &terminal)?;
    runner.start()
}

fn print_and_flush(buffer: &mut impl Write, content: &str) -> io::Result<()> {
    buffer.write_all(content.as_bytes())?;
    buffer.flush()
}

#[derive(Debug)]
enum State {
    Ready,
    QueryChanged,
    PathsChanged,
    SelectionChanged,
    Invoke(MatchedPath),
}

struct Runner<'a, W: Write, T: Terminal> {
    preferences: &'a Preferences,
    stdout: &'a mut W,
    terminal: &'a T,
    repo: Option<Repository>,
    clipboard: Option<ClipboardContext>,
    max_columns: u16,
    max_rows: u16,
    starting_point: StartingPoint,
    query: Query,
    selection: u16,
    state: State,
}

impl<'a, W: Write, T: Terminal> Runner<'a, W, T> {
    fn new(preferences: &'a Preferences, stdout: &'a mut W, terminal: &'a T) -> Result<Self> {
        let repo = if preferences.gitignore {
            match Repository::discover(&preferences.starting_point) {
                Ok(r) => Some(r),
                Err(_) => {
                    log::info!(
                        "The starting point `{}` is not a Git repository",
                        &preferences.starting_point
                    );
                    None
                }
            }
        } else {
            None
        };

        let clipboard = match ClipboardContext::new().map_err(|e| Error::clipboard(e)) {
            Ok(c) => Some(c),
            Err(e) => {
                log::warn!("Failed to initialize clipboard: {}", e);
                None
            }
        };
        let (max_columns, max_rows) = terminal.size()?;
        let starting_point = StartingPoint::new(&preferences.starting_point)?;
        let query = Query::new(&preferences.query);

        Ok(Self {
            preferences,
            stdout,
            terminal,
            repo,
            clipboard,
            max_columns,
            max_rows,
            starting_point,
            query,
            selection: 0,
            state: State::QueryChanged,
        })
    }

    fn start(&'a mut self) -> Result<()> {
        let mut paths = self.find_paths(self.paths_rows(self.max_rows))?;
        self.start_writing()?;

        loop {
            log::trace!("state={:?}", self.state);
            match self.state {
                State::QueryChanged => self.write_result(paths.as_ref())?,
                State::PathsChanged => {
                    self.write_result(paths.as_ref())?;
                    self.state = State::Ready;
                }
                State::SelectionChanged => {
                    self.write_result(paths.as_ref())?;
                    self.state = State::Ready;
                }
                _ => (),
            }

            if self.terminal.poll(Duration::from_millis(300))? {
                let ev = self.terminal.read()?;
                log::trace!("Event={:?}", ev);
                if ev == ctrl!('c') || ev == esc!() {
                    break;
                } else if let char_extracted!(c) = ev {
                    self.query.push(c);
                    self.state = State::QueryChanged;
                } else if ev == backspace!() {
                    self.query.pop();
                    self.state = State::QueryChanged;
                } else if ev == up!() || ev == ctrl!('p') {
                    if self.selection > 0 {
                        self.selection -= 1;
                        self.state = State::SelectionChanged;
                    }
                } else if ev == down!() || ev == ctrl!('n') {
                    if (self.selection as usize) < paths.len() - 1 {
                        self.selection += 1;
                        self.state = State::SelectionChanged;
                    }
                } else if ev == enter!() {
                    let path = paths.get(self.selection as usize).unwrap(); // TODO: Do not use unwrap()
                    self.state = State::Invoke(path.clone());
                    break;
                } else if ev == ctrl!('y') {
                    let path: &MatchedPath = paths.get(self.selection as usize).unwrap(); // TODO: Do not use unwrap()
                    if let Some(c) = self.clipboard.as_mut() {
                        c.set_contents(path.absolute().to_owned())
                            .map_err(|e| Error::clipboard(e))?;
                    }
                } else if ev == ctrl!('d') {
                    let path: &MatchedPath = paths.get(self.selection as usize).unwrap(); // TODO: Do not use unwrap()
                    if let Some(c) = self.clipboard.as_mut() {
                        c.set_contents(path.relative().to_owned())
                            .map_err(|e| Error::clipboard(e))?;
                    }
                } else if let Event::Resize(c, r) = ev {
                    self.max_columns = c;
                    self.max_rows = r;
                    self.selection = if self.selection > r {
                        self.paths_rows(r) - 1
                    } else {
                        self.selection
                    };
                    paths = self.find_paths(self.paths_rows(self.max_rows))?;
                    self.state = State::PathsChanged;
                }
            } else if let State::QueryChanged = self.state {
                paths = self.find_paths(self.paths_rows(self.max_rows))?;
                self.state = State::PathsChanged;
                self.selection = 0;
            }
        }

        self.finish_writing()?;

        if let State::Invoke(ref path) = self.state {
            invoke(&Libc, self.preferences, path.absolute())?;
        }
        Ok(())
    }

    fn find_paths(&self, limit: impl Into<usize>) -> Result<Vec<MatchedPath>> {
        let mut paths = Vec::with_capacity(100); // TODO: Tune this capacity later.

        for path in Finder::new(
            &self.starting_point,
            &self.query.to_string(),
            self.repo.as_ref(),
        )? {
            match path {
                Ok(path) => paths.push(path),
                Err(e) => log::error!("Failed to get the path: {}", e),
            }
        }

        paths.sort();
        Ok(paths.into_iter().take(limit.into()).collect())
    }

    fn paths_rows(&self, row: impl Into<u16>) -> u16 {
        // TODO: raise an error when the number of rows is too small.
        match self.preferences.status_line {
            StatusLine::None => row.into() - 2,
            _ => row.into() - 3,
        }
    }

    fn max_path_width(&self) -> usize {
        (self.max_columns - 2).into()
    }

    fn write_result(&mut self, paths: &[MatchedPath]) -> Result<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(0, 0),
            Clear(ClearType::FromCursorDown),
            style::Print("Search: "),
            style::Print(&self.query),
            cursor::SavePosition,
            cursor::MoveToNextLine(1),
        )?;
        let mut selected: Option<&MatchedPath> = None;
        for (idx, path) in paths.iter().enumerate() {
            let idx = idx as u16;
            let prefix = if idx == self.selection {
                selected = Some(path);
                "> "
            } else {
                "  "
            };
            queue!(self.stdout, style::Print(prefix))?;

            for chunk in path.relative_chunks(self.max_path_width()) {
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

        let selected = match (selected, &self.preferences.status_line) {
            (_, StatusLine::None) => None,
            (None, _) => Some("No matching files found.".to_string()),
            (Some(s), StatusLine::Absolute) => {
                Some(s.truncated_absolute(self.max_columns as usize))
            }
            (Some(s), StatusLine::Relative) => {
                Some(s.truncated_relative(self.max_columns as usize))
            }
        };
        if let Some(ref s) = selected {
            queue!(self.stdout, cursor::MoveToRow(self.max_rows - 2))?;
            self.write_status_line(s)?;
        } else {
            queue!(self.stdout, cursor::MoveToRow(self.max_rows))?;
        }
        self.write_help_line()?;
        queue!(self.stdout, cursor::RestorePosition)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn start_writing(&mut self) -> Result<()> {
        queue!(self.stdout, EnterAlternateScreen, style::ResetColor)?;
        self.stdout.flush()?;
        self.terminal.enable_raw_mode()?;
        log::debug!(
            "Raw mode enabled with terminal size(columns={}, rows={})",
            self.max_columns,
            self.max_rows
        );
        Ok(())
    }

    fn finish_writing(&mut self) -> Result<()> {
        execute!(self.stdout, LeaveAlternateScreen)?;
        self.terminal.disable_raw_mode()?;
        log::debug!(
            "Initialized terminal! size=({:?}, {:?})",
            self.max_columns,
            self.max_rows,
        );
        Ok(())
    }

    fn write_status_line(&mut self, selected: &str) -> Result<()> {
        queue!(
            self.stdout,
            style::SetAttribute(Attribute::Bold),
            style::SetAttribute(Attribute::Reverse),
            style::Print(format!(
                "{:width$}",
                selected,
                width = self.max_columns as usize
            )),
            style::SetAttribute(Attribute::Reset),
        )?;
        Ok(())
    }

    fn write_help_line(&mut self) -> Result<()> {
        // TODO: This number is the same amount of columns occupied by this status line.
        if self.max_columns < 97 {
            return Ok(());
        }
        queue!(
            self.stdout,
            cursor::MoveToNextLine(1),
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
}
