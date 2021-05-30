use std::env::ArgsOs;
use std::io::{self, Stderr, Stdout, Write};
use std::process::exit;
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
use crate::Error;

pub fn safe_exit(code: i32, stdout: Stdout, stderr: Stderr) {
    let _ = stdout.lock().flush();
    let _ = stderr.lock().flush();
    exit(code)
}

pub fn entrypoint(args: ArgsOs, stdout: &mut impl Write, stderr: &mut impl Write) -> Result<()> {
    let args = Parser::new(args).parse()?;
    if args.help {
        print_help(stdout)?;
        return Ok(());
    }

    let (_, mut rows) = terminal::size()?;
    let mut query = args.query;
    let mut paths = find_paths(&args.starting_point, &query, rows - 1)?;
    let mut selection: u16 = 0;
    let mut state = State::QueryChanged;
    initialize_terminal(stdout)?;

    loop {
        match state {
            State::QueryChanged => output_on_terminal(stdout, &query, &paths[..], selection)?,
            State::PathsChanged => {
                output_on_terminal(stdout, &query, &paths[..], selection)?;
                state = State::Ready;
            }
            State::SelectionChanged => {
                output_on_terminal(stdout, &query, &paths[..], selection)?;
                state = State::Ready;
            }
            _ => (),
        }

        if event::poll(Duration::from_millis(300))? {
            let ev = event::read()?;
            // TODO: Support uppercase
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
            }) = ev
            {
                query.push(c);
                state = State::QueryChanged;
            } else if ev == Event::Key(KeyCode::Backspace.into()) {
                query.pop();
                state = State::QueryChanged;
            } else if ev == Event::Key(KeyCode::Up.into()) {
                if selection > 0 {
                    selection -= 1;
                }
                state = State::SelectionChanged;
            } else if ev == Event::Key(KeyCode::Down.into()) {
                if selection < (rows - 1) {
                    selection += 1;
                }
                state = State::SelectionChanged;
            } else if ev == Event::Key(KeyCode::Enter.into()) {
                let path: &MatchedPath = paths.get(selection as usize).unwrap(); // TODO: Do not use unwrap()
                state = State::Invoke(&path);
                break;
            } else if ev
                == Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                })
                || ev == Event::Key(KeyCode::Esc.into())
            {
                // CTRL-C does not send SIGINT even on UNIX/Linux because it's in Raw mode.
                // Also, we handle Esc as the same.
                break;
            } else if let Event::Resize(_, r) = ev {
                rows = r;
                state = State::PathsChanged;
            }
        } else if let State::QueryChanged = state {
            paths = find_paths(&args.starting_point, &query, rows - 1)?;
            state = State::PathsChanged;
            selection = 0;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let State::Invoke(path) = state {
        // TODO: Decide which we should pass: relative or absolute.
        let path = path.relative();
        let output = invoke(&args.exec, path)?;
        stdout.write_all(&output.stdout)?;
        stderr.write_all(&output.stderr)?;
        if !output.status.success() {
            return Err(Error::exec(&format!(
                "Failed to execute command `{} {}`",
                args.exec, path,
            )));
        }
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

fn print_help(buffer: &mut impl Write) -> io::Result<()> {
    buffer.write_all(format!("{}\n", HELP).as_bytes())?;
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
        queue!(stdout, style::Print(format!("{}", prefix)))?;
        for chunk in path.chunks() {
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

fn find_paths(starting_point: &str, query: &str, limit: u16) -> Result<Vec<MatchedPath>> {
    let mut paths = Vec::with_capacity(100); // TODO: Tune this capacity later.

    for path in Finder::new(starting_point, query)? {
        // TODO: Shouldn't we stop iteration when some paths retuns Err?
        let path = path?;
        paths.push(path);
    }

    paths.sort();
    Ok(paths.into_iter().take(limit.into()).collect())
}

fn invoke(exec: &str, path: &str) -> Result<std::process::Output> {
    let mut exec = exec.split_whitespace();
    let program = exec
        .next()
        .ok_or_else(|| Error::args("\"exec\" cannot be processed with empty string"))?;
    let mut command = std::process::Command::new(program);
    for arg in exec {
        command.arg(arg);
    }
    command.arg(path);
    let output = command.output()?;
    Ok(output)
}
