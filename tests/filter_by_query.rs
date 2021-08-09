use std::ffi::OsString;

use crossterm::event::{Event, KeyCode};

use helper::{create_tree, MockTerminal, MockTerminalEvent};
use thwack::entrypoint;

mod helper;

#[test]
#[cfg(not(target_os = "windows"))]
fn show_all_as_many_as_the_size_of_terminal_without_query() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative"
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer,
        buf!(
            b"\x1b[?1049h\x1b[0m\x1b[1;1H\x1b[JSearch: \x1b7\x1b[1E\
            >\x20.browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\
            \x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[?1049l"
        )
    );
}

#[test]
#[cfg(target_os = "windows")]
fn show_all_as_many_as_the_size_of_terminal_without_query() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative"
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer,
        buf!(
            b"\x1b[?1049h\x1b[0m\x1b[1;1H\x1b[JSearch: \x1b7\x1b[1E\
            >\x20.browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config\\bar.toml\x1b[1E\
            \x20\x20.config\\ok.toml\x1b[1E\
            \x20\x20lib\\bar.js\x1b[1E\
            \x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[?1049l"
        )
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn show_filtered_paths_with_query() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative",
        "browser",
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer,
        buf!(
            b"\x1b[?1049h\x1b[0m\x1b[1;1H\x1b[JSearch: browser\x1b7\x1b[1E\
            >\x20.\x1b[1mbrowse\x1b[0mrslist\x1b[1mr\x1b[0mc\x1b[1E\
            \x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[?1049l"
        )
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn show_filtered_paths_with_query_interactively() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative",
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Char('b').into())));
    event.add(Some(Event::Key(KeyCode::Char('r').into())));
    event.add(Some(Event::Key(KeyCode::Char('o').into())));
    event.add(Some(Event::Key(KeyCode::Char('w').into())));
    event.add(Some(Event::Key(KeyCode::Char('s').into())));
    event.add(Some(Event::Key(KeyCode::Char('e').into())));
    event.add(Some(Event::Key(KeyCode::Char('r').into())));
    event.add(None);
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer,
        buf!(
            b"\x1b[?1049h\x1b[0m\x1b[1;1H\x1b[JSearch: \x1b7\x1b[1E\
            >\x20.browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: b\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: br\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: bro\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: brow\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: brows\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\x1b[JSearch: browse\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: browser\x1b7\x1b[1E> .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: browser\x1b7\x1b[1E> .\x1b[1mbrowse\x1b[0mrslist\x1b[1mr\x1b[0mc\x1b[1E\
            \x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[?1049l"
        )
    );
}

#[test]
#[cfg(not(target_os = "windows"))]
fn show_filtered_paths_with_query_interactively_including_backspace() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative",
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Char('B').into())));
    event.add(Some(Event::Key(KeyCode::Char('r').into())));
    event.add(None);
    event.add(Some(Event::Key(KeyCode::Backspace.into())));
    event.add(None);
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer,
        buf!(
            b"\x1b[?1049h\x1b[0m\x1b[1;1H\
            \x1b[JSearch: \x1b7\x1b[1E\
            > .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: B\x1b7\x1b[1E\
            > .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: Br\x1b7\x1b[1E\
            > .browserslistrc\x1b[1E\
            \x20\x20.editorconfig\x1b[1E\
            \x20\x20.env\x1b[1E\
            \x20\x20.env.local\x1b[1E\
            \x20\x20.gitignore\x1b[1E\
            \x20\x20.npmrc\x1b[1E\
            \x20\x20.nvmrc\x1b[1E\
            \x20\x20Dockerfile\x1b[1E\
            \x20\x20LICENSE\x1b[1E\
            \x20\x20README.md\x1b[1E\
            \x20\x20package-lock.json\x1b[1E\
            \x20\x20package.json\x1b[1E\
            \x20\x20tsconfig.json\x1b[1E\
            \x20\x20\xe2\x98\x95.txt\x1b[1E\
            \x20\x20.config/bar.toml\x1b[1E\
            \x20\x20.config/ok.toml\x1b[1E\
            \x20\x20lib/bar.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: Br\x1b7\x1b[1E\
            > .config/\x1b[1mb\x1b[0ma\x1b[1mr\x1b[0m.toml\x1b[1E\
            \x20\x20lib/\x1b[1mb\x1b[0ma\x1b[1mr\x1b[0m.js\x1b[1E\
            \x20\x20.\x1b[1mb\x1b[0mrowserslist\x1b[1mr\x1b[0mc\x1b[1E\x1b[19d\x1b[1m\x1b[7m.config/bar.toml\
            \x20\x20                                                              \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: B\x1b7\x1b[1E\
            > .config/\x1b[1mb\x1b[0ma\x1b[1mr\x1b[0m.toml\x1b[1E\
            \x20\x20lib/\x1b[1mb\x1b[0ma\x1b[1mr\x1b[0m.js\x1b[1E\
            \x20\x20.\x1b[1mb\x1b[0mrowserslist\x1b[1mr\x1b[0mc\x1b[1E\x1b[19d\x1b[1m\x1b[7m.config/bar.toml                                                                \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[1;1H\
            \x1b[JSearch: B\x1b7\x1b[1E\
            > .\x1b[1mb\x1b[0mrowserslistrc\x1b[1E\
            \x20\x20.config/\x1b[1mb\x1b[0mar.toml\x1b[1E\
            \x20\x20lib/\x1b[1mb\x1b[0mar.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/df.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/ok.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/a/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/util.js\x1b[1E\
            \x20\x20lib/a/\x1b[1mb\x1b[0m/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/y/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/y/util.js\x1b[1E\
            \x20\x20src/a/\x1b[1mb\x1b[0m/index.js\x1b[1E\
            \x20\x20lib/a/\x1b[1mb\x1b[0m/c/index.js\x1b[1E\
            \x20\x20lib/a/\x1b[1mb\x1b[0m/c/\xe2\x98\x95.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/y/z/index.js\x1b[1E\
            \x20\x20li\x1b[1mb\x1b[0m/x/y/z/util.js\x1b[1E\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                 \x1b[0m\x1b[1E\
            \x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b8\x1b[?1049l"
        )
    );
}
