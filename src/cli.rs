use std::env::ArgsOs;
use std::ffi::CString;
use std::io::{self, Stderr, Stdout, Write};
use std::os::raw::c_char;
use std::process::exit;
use std::ptr;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{self, Attribute},
    terminal::{self, ClearType},
};

use crate::args::{Parser, HELP};
use crate::error::Result;
use crate::finder::Finder;
use crate::matched_path::MatchedPath;
use crate::starting_point::StartingPoint;
use crate::Error;

pub fn safe_exit(code: i32, stdout: Stdout, stderr: Stderr) {
    let _ = stdout.lock().flush();
    let _ = stderr.lock().flush();
    exit(code)
}

pub fn entrypoint(args: ArgsOs, stdout: &mut impl Write) -> Result<()> {
    let args = Parser::new(args).parse()?;
    if args.help {
        print_and_flush(stdout, HELP)?;
        return Ok(());
    }
    if args.version {
        print_and_flush(
            stdout,
            &format!("{} {}\n", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        )?;
        return Ok(());
    }

    let (mut columns, mut rows) = terminal::size()?;
    let starting_point = StartingPoint::new(args.starting_point)?;
    let mut query = args.query;
    let mut paths = find_paths(&starting_point, &query, paths_rows(rows))?;
    let mut selection: u16 = 0;
    let mut state = State::QueryChanged;
    initialize_terminal(stdout)?;

    loop {
        match state {
            State::QueryChanged => {
                output_on_terminal(stdout, &query, &paths[..], selection, columns)?
            }
            State::PathsChanged => {
                output_on_terminal(stdout, &query, &paths[..], selection, columns)?;
                state = State::Ready;
            }
            State::SelectionChanged => {
                output_on_terminal(stdout, &query, &paths[..], selection, columns)?;
                state = State::Ready;
            }
            _ => (),
        }

        if event::poll(Duration::from_millis(300))? {
            let ev = event::read()?;
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
                }
                state = State::SelectionChanged;
            } else if should_move_down(&ev) {
                if selection < paths_rows(rows) - 1 {
                    selection += 1;
                }
                state = State::SelectionChanged;
            } else if ev == Event::Key(KeyCode::Enter.into()) {
                let path: &MatchedPath = paths.get(selection as usize).unwrap(); // TODO: Do not use unwrap()
                state = State::Invoke(&path);
                break;
            } else if let Event::Resize(c, r) = ev {
                columns = c;
                rows = r;
                paths = find_paths(&starting_point, &query, paths_rows(rows))?;
                state = State::PathsChanged;
            }
        } else if let State::QueryChanged = state {
            paths = find_paths(&starting_point, &query, paths_rows(rows))?;
            state = State::PathsChanged;
            selection = 0;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let State::Invoke(path) = state {
        invoke(&args.exec, path.absolute())?;
    }
    Ok(())
}

enum State<'a> {
    Ready,
    QueryChanged,
    PathsChanged,
    SelectionChanged,
    Invoke(&'a MatchedPath),
}

fn paths_rows(row: u16) -> u16 {
    // TODO: raise an error when the number of rows is too small.
    row - 1
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

fn initialize_terminal(stdout: &mut impl Write) -> Result<()> {
    queue!(stdout, terminal::EnterAlternateScreen, style::ResetColor,)?;
    stdout.flush()?;
    terminal::enable_raw_mode()?;
    Ok(())
}

fn output_on_terminal(
    stdout: &mut impl Write,
    query: &str,
    paths: &[MatchedPath],
    selection: u16,
    max_columns: u16,
) -> Result<()> {
    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::FromCursorDown),
        style::Print("Search: "),
        style::Print(query),
        cursor::SavePosition,
        cursor::MoveToNextLine(1),
    )?;
    for (idx, path) in paths.iter().enumerate() {
        let idx = idx as u16;
        let prefix = if idx == selection { "> " } else { "  " };
        queue!(stdout, style::Print(prefix))?;

        let max_columns: usize = max_columns as usize - 2; // The prefix "> " requires 2 columns.
        for chunk in path.relative_chunks(max_columns) {
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
    queue!(stdout, cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn find_paths(starting_point: &StartingPoint, query: &str, limit: u16) -> Result<Vec<MatchedPath>> {
    let mut paths = Vec::with_capacity(100); // TODO: Tune this capacity later.

    for path in Finder::new(starting_point, query)? {
        // TODO: Shouldn't we stop iteration when some paths returns Err?
        let path = path?;
        paths.push(path);
    }

    paths.sort();
    Ok(paths.into_iter().take(limit.into()).collect())
}

/// Invoke the specified command with the selected path.
fn invoke(exec: &str, path: &str) -> Result<()> {
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
