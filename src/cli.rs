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

    // let (columns, rows) = terminal::size()?;
    // println!("columns={:?}, rows={:?}", columns, rows); // TODO: Use rows to limit the results

    let mut query = args.query;
    let mut paths = find_paths(&args.starting_point, &query)?;
    let mut state = State::QueryChanged;
    initialize_terminal(&mut stdout)?;

    loop {
        match state {
            State::Ready => (),
            State::QueryChanged => output_on_terminal(&mut stdout, &query, &paths[..])?,
            State::PathsChanged => {
                output_on_terminal(&mut stdout, &query, &paths[..])?;
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
            }
        } else {
            match state {
                State::QueryChanged => {
                    paths = find_paths(&args.starting_point, &query)?;
                    state = State::PathsChanged;
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

fn output_on_terminal(stdout: &mut impl Write, query: &str, paths: &[MatchedPath]) -> Result<()> {
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
    for path in paths {
        queue!(
            stdout,
            style::Print(format!("{}", path)),
            cursor::MoveToNextLine(1),
        )?;
    }
    queue!(stdout, cursor::RestorePosition)?;
    stdout.flush()?;
    Ok(())
}

fn find_paths(starting_point: &str, query: &str) -> Result<Vec<MatchedPath>> {
    let mut paths = vec![];
    for path in Finder::new(starting_point, query)? {
        let path = path?;
        paths.push(path);
    }
    Ok(paths)
}
