use std::env::ArgsOs;
use std::io::{self, Write};
use std::process::exit;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue, style,
    terminal::{self, ClearType},
};

use crate::args::{Parser, HELP};
use crate::error::Result;
use crate::finder::Finder;
use crate::MatchedPath;

pub fn safe_exit(code: i32) {
    let _ = std::io::stdout().lock().flush();
    let _ = std::io::stderr().lock().flush();
    exit(code)
}

pub fn entrypoint(args: ArgsOs, mut stdout: impl Write) -> Result<()> {
    let args = Parser::new(args).parse()?;
    if args.help {
        print_help(&mut stdout)?;
        return Ok(());
    }

    let (_, mut rows) = terminal::size()?;
    let mut query = args.query;
    let mut paths = find_paths(&args.starting_point, &query, rows - 1)?;
    let mut selection: u16 = 0;
    let mut state = State::QueryChanged;
    initialize_terminal(&mut stdout)?;

    loop {
        match state {
            State::Ready => (),
            State::QueryChanged => output_on_terminal(&mut stdout, &query, &paths[..], selection)?,
            State::PathsChanged => {
                output_on_terminal(&mut stdout, &query, &paths[..], selection)?;
                state = State::Ready;
            }
            State::SelectionChanged => {
                output_on_terminal(&mut stdout, &query, &paths[..], selection)?;
                state = State::Ready;
            }
        }

        if event::poll(Duration::from_millis(300))? {
            let ev = event::read()?;
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
            } else if ev
                == Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                })
            {
                // CTRL-C does not send SIGINT even on UNIX/Linux because it's in Raw mode.
                break;
            } else if ev == Event::Key(KeyCode::Esc.into()) {
                break;
            } else if let Event::Resize(_, r) = ev {
                rows = r;
                state = State::PathsChanged;
            }
        } else {
            match state {
                State::QueryChanged => {
                    paths = find_paths(&args.starting_point, &query, rows - 1)?;
                    state = State::PathsChanged;
                    selection = 0;
                }
                _ => (),
            }
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

enum State {
    Ready,
    QueryChanged,
    PathsChanged,
    SelectionChanged,
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
    std::fs::File::create("/tmp/ok.txt")?.write_all(format!("{:?}", paths).as_bytes())?;
    for (idx, path) in paths.iter().enumerate() {
        let idx = idx as u16;
        let prefix = if idx == selection { "> " } else { "  " };
        queue!(
            stdout,
            style::Print(format!("{}{}", prefix, path)),
            cursor::MoveToNextLine(1),
        )?;
    }
    queue!(stdout, cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn find_paths(starting_point: &str, query: &str, limit: u16) -> Result<Vec<MatchedPath>> {
    let mut paths = vec![];
    // TODO: Sort
    for path in Finder::new(starting_point, query)?.take(limit.into()) {
        let path = path?;
        paths.push(path);
    }
    Ok(paths)
}
