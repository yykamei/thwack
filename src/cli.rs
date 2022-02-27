use std::ffi::{CString, OsString};
use std::io::{self, Stderr, Stdout, Write};
use std::os::raw::c_char;
use std::process::exit;
use std::ptr;
use std::time::Duration;

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
use crate::matched_path::MatchedPath;
use crate::preferences::Preferences;
use crate::starting_point::StartingPoint;
use crate::status_line::StatusLine;
use crate::terminal::{Terminal, TerminalEvent};
use crate::{logger, Error};

pub fn safe_exit(code: i32, stdout: Stdout, stderr: Stderr) {
    let _ = stdout.lock().flush();
    let _ = stderr.lock().flush();
    exit(code)
}

pub fn entrypoint<A: Iterator<Item = OsString>, W: Write>(
    args: A,
    stdout: &mut W,
    terminal: impl Terminal,
    mut event: impl TerminalEvent,
) -> Result<()> {
    let args = Args::new(args).parse()?;
    if let Some(ref path) = args.log_file {
        logger::init(path)?;
        log::debug!("Logger initialized!");
    }
    if args.help {
        print_and_flush(stdout, HELP)?;
        log::debug!("Show help and exit");
        return Ok(());
    }
    if args.version {
        print_and_flush(
            stdout,
            &format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )?;
        log::debug!("Show version and exit");
        return Ok(());
    }

    let repo = if args.gitignore {
        match Repository::discover(&args.starting_point) {
            Ok(r) => Some(r),
            Err(_) => {
                log::info!(
                    "The starting point `{}` is not a Git repository",
                    &args.starting_point
                );
                None
            }
        }
    } else {
        None
    };
    let (mut columns, mut rows) = terminal.size()?;
    let starting_point = StartingPoint::new(&args.starting_point)?;
    let mut query = args.query.clone();
    let mut paths = find_paths(
        &starting_point,
        &query,
        paths_rows(&args, rows),
        repo.as_ref(),
    )?;
    let mut selection: u16 = 0;
    let mut state = State::QueryChanged;
    initialize_terminal(stdout, &terminal)?;
    log::debug!(
        "Initialized terminal! size=({:?}, {:?}), starting_point={:?}",
        columns,
        rows,
        starting_point
    );

    loop {
        log::trace!("state={:?}", state);
        match state {
            State::QueryChanged => {
                output_on_terminal(stdout, &args, &query, &paths[..], selection, columns, rows)?
            }
            State::PathsChanged => {
                output_on_terminal(stdout, &args, &query, &paths[..], selection, columns, rows)?;
                state = State::Ready;
            }
            State::SelectionChanged => {
                output_on_terminal(stdout, &args, &query, &paths[..], selection, columns, rows)?;
                state = State::Ready;
            }
            _ => (),
        }

        if event.poll(Duration::from_millis(300))? {
            let ev = event.read()?;
            log::trace!("Event={:?}", ev);
            if should_just_exit(&ev) {
                break;
            } else if let Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
            }) = ev
            {
                query.push(c);
                state = State::QueryChanged;
            } else if ev == Event::Key(KeyCode::Backspace.into()) {
                query.pop();
                state = State::QueryChanged;
            } else if should_move_up(&ev) {
                if selection > 0 {
                    selection -= 1;
                    state = State::SelectionChanged;
                }
            } else if should_move_down(&ev) {
                if (selection as usize) < paths.len() - 1 {
                    selection += 1;
                    state = State::SelectionChanged;
                }
            } else if ev == Event::Key(KeyCode::Enter.into()) {
                let path: &MatchedPath = paths.get(selection as usize).unwrap(); // TODO: Do not use unwrap()
                state = State::Invoke(path);
                break;
            } else if let Event::Resize(c, r) = ev {
                columns = c;
                rows = r;
                selection = if selection > r {
                    paths_rows(&args, r) - 1
                } else {
                    selection
                };
                paths = find_paths(
                    &starting_point,
                    &query,
                    paths_rows(&args, rows),
                    repo.as_ref(),
                )?;
                state = State::PathsChanged;
            }
        } else if let State::QueryChanged = state {
            paths = find_paths(
                &starting_point,
                &query,
                paths_rows(&args, rows),
                repo.as_ref(),
            )?;
            state = State::PathsChanged;
            selection = 0;
        }
    }

    execute!(stdout, LeaveAlternateScreen)?;
    terminal.disable_raw_mode()?;

    if let State::Invoke(path) = state {
        invoke(&args.exec, path.absolute())?;
    }
    Ok(())
}

#[derive(Debug)]
enum State<'a> {
    Ready,
    QueryChanged,
    PathsChanged,
    SelectionChanged,
    Invoke(&'a MatchedPath),
}

fn paths_rows(args: &Preferences, row: u16) -> u16 {
    // TODO: raise an error when the number of rows is too small.
    match args.status_line {
        StatusLine::None => row - 2,
        _ => row - 3,
    }
}

fn should_just_exit(ev: &Event) -> bool {
    // CTRL-C does not send SIGINT even on UNIX/Linux because it's in Raw mode.
    // Also, we handle Esc as the same.
    ev == &Event::Key(KeyEvent {
        code: KeyCode::Char('c'),
        modifiers: KeyModifiers::CONTROL,
    }) || ev == &Event::Key(KeyCode::Esc.into())
}

fn should_move_up(ev: &Event) -> bool {
    ev == &Event::Key(KeyCode::Up.into())
        || ev
            == &Event::Key(KeyEvent {
                code: KeyCode::Char('p'),
                modifiers: KeyModifiers::CONTROL,
            })
}

fn should_move_down(ev: &Event) -> bool {
    ev == &Event::Key(KeyCode::Down.into())
        || ev
            == &Event::Key(KeyEvent {
                code: KeyCode::Char('n'),
                modifiers: KeyModifiers::CONTROL,
            })
}

fn print_and_flush(buffer: &mut impl Write, content: &str) -> io::Result<()> {
    buffer.write_all(content.as_bytes())?;
    buffer.flush()
}

fn initialize_terminal(stdout: &mut impl Write, terminal: &impl Terminal) -> Result<()> {
    queue!(stdout, EnterAlternateScreen, style::ResetColor)?;
    stdout.flush()?;
    terminal.enable_raw_mode()?;
    Ok(())
}

fn output_on_terminal(
    stdout: &mut impl Write,
    args: &Preferences,
    query: &str,
    paths: &[MatchedPath],
    selection: u16,
    max_columns: u16,
    max_rows: u16,
) -> Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown),
        style::Print("Search: "),
        style::Print(query),
        cursor::SavePosition,
        cursor::MoveToNextLine(1),
    )?;

    let max_path_width: usize = max_columns as usize - 2; // The prefix "> " requires 2 columns.
    let mut selected: Option<&MatchedPath> = None;
    for (idx, path) in paths.iter().enumerate() {
        let idx = idx as u16;
        let prefix = if idx == selection {
            selected = Some(path);
            "> "
        } else {
            "  "
        };
        queue!(stdout, style::Print(prefix))?;

        for chunk in path.relative_chunks(max_path_width) {
            if chunk.matched() {
                queue!(
                    stdout,
                    style::SetAttribute(Attribute::Bold),
                    style::Print(format!("{}", chunk)),
                    style::SetAttribute(Attribute::Reset),
                )?;
            } else {
                queue!(stdout, style::Print(format!("{}", chunk)))?;
            }
        }
        queue!(stdout, cursor::MoveToNextLine(1))?;
    }

    let selected = match (selected, &args.status_line) {
        (_, StatusLine::None) => None,
        (None, _) => Some("No matching files found.".to_string()),
        (Some(s), StatusLine::Absolute) => Some(s.truncated_absolute(max_columns as usize)),
        (Some(s), StatusLine::Relative) => Some(s.truncated_relative(max_columns as usize)),
    };
    if let Some(ref s) = selected {
        queue!(stdout, cursor::MoveToRow(max_rows - 1))?;
        status_line(stdout, max_columns as usize, s)?;
    } else {
        queue!(stdout, cursor::MoveToRow(max_rows))?;
    }
    help_line(stdout, max_columns as usize)?;
    queue!(stdout, cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn status_line(stdout: &mut impl Write, max_columns: usize, selected: &str) -> Result<()> {
    queue!(
        stdout,
        style::SetAttribute(Attribute::Bold),
        style::SetAttribute(Attribute::Reverse),
        style::Print(format!("{:width$}", selected, width = max_columns)),
        style::SetAttribute(Attribute::Reset),
    )?;
    Ok(())
}

fn help_line(stdout: &mut impl Write, max_columns: usize) -> Result<()> {
    // TODO: This number is the same amount of columns occupied by this status line.
    if max_columns < 58 {
        return Ok(());
    }
    queue!(
        stdout,
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
    )?;
    Ok(())
}

fn find_paths(
    starting_point: &StartingPoint,
    query: &str,
    limit: u16,
    repo: Option<&Repository>,
) -> Result<Vec<MatchedPath>> {
    let mut paths = Vec::with_capacity(100); // TODO: Tune this capacity later.

    for path in Finder::new(starting_point, query, repo)? {
        // TODO: Shouldn't we stop iteration when some paths returns Err?
        match path {
            Ok(path) => paths.push(path),
            Err(e) => log::error!("Failed to get the path: {}", e),
        }
    }

    paths.sort();
    Ok(paths.into_iter().take(limit.into()).collect())
}

/// Invoke the specified command with the selected path.
fn invoke(exec: &str, path: &str) -> Result<()> {
    // TODO: This function is not tested because execvp(3) replaces the current process with a new process, which means tests will stop.

    let mut cstrings: Vec<CString> = Vec::with_capacity(10); // TODO: Why is it 10?
    for arg in exec.split_whitespace() {
        cstrings.push(CString::new(arg)?);
    }
    cstrings.push(CString::new(path)?);
    let argv: Vec<*const c_char> = cstrings
        .iter()
        .map(|c| c.as_ptr())
        .chain(std::iter::once(ptr::null()))
        .collect();

    let errno = unsafe { libc::execvp(cstrings[0].as_ptr(), argv.as_ptr()) };

    Err(Error::exec(&format!(
        "`{} {}` failed and returned {}",
        exec, path, errno
    )))
}
