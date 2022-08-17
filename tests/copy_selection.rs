use std::ffi::OsString;

use copypasta::{ClipboardContext, ClipboardProvider};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

use helper::{create_tree, MockTerminal, MockTerminalEvent};
use thwack::entrypoint;

mod helper;

#[cfg(not(target_os = "linux"))]
#[test]
fn copy_with_absolute_path() {
    let dir = create_tree().unwrap();
    let args = args![
        "thwack",
        "--starting-point",
        dir.path().to_str().unwrap(),
        "--status-line=relative"
    ];
    let mut event = MockTerminalEvent::new();
    event.add(Some(Event::Key(KeyCode::Down.into())));
    event.add(Some(Event::Key(KeyEvent {
        code: KeyCode::Char('d'),
        modifiers: KeyModifiers::CONTROL,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    })));
    event.add(Some(Event::Key(KeyCode::Esc.into())));
    let mut buffer = buf!();
    let result = entrypoint(args, &mut buffer, MockTerminal, event);
    assert!(result.is_ok());
    assert_eq!(
        buffer.normalize_path(),
        buf!(
            b"\x1b[?1049h\x1b[0m",
            b"\x1b[1;1H\x1b[J",
            b"Search: \x1b7\x1b[1E",
            b"> .browserslistrc\x1b[1E",
            b"  .config/bar.toml\x1b[1E",
            b"  .config/ok.toml\x1b[1E",
            b"  .editorconfig\x1b[1E",
            b"  .env\x1b[1E",
            b"  .env.local\x1b[1E",
            b"  .gitignore\x1b[1E",
            b"  .npmrc\x1b[1E",
            b"  .nvmrc\x1b[1E",
            b"  Dockerfile\x1b[1E",
            b"  LICENSE\x1b[1E",
            b"  README.md\x1b[1E",
            b"  lib/a/b/c/index.js\x1b[1E",
            b"  lib/a/b/c/\xe2\x98\x95.js\x1b[1E",
            b"  lib/a/b/index.js\x1b[1E",
            b"  lib/a/index.js\x1b[1E",
            b"  lib/bar.js\x1b[1E",
            b"\x1b[19d\x1b[1m\x1b[7m.browserslistrc                                                                                   \x1b[0m\x1b[1E",
            b"\x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b[2C\x1b[1m<C-d>/<C-y>:\x1b[0m\x1b[1CCopy (relative/absolute)",
            b"\x1b8",
            b"\x1b[1;1H\x1b[J",
            b"Search: \x1b7\x1b[1E",
            b"  .browserslistrc\x1b[1E",
            b"> .config/bar.toml\x1b[1E",
            b"  .config/ok.toml\x1b[1E",
            b"  .editorconfig\x1b[1E",
            b"  .env\x1b[1E",
            b"  .env.local\x1b[1E",
            b"  .gitignore\x1b[1E",
            b"  .npmrc\x1b[1E",
            b"  .nvmrc\x1b[1E",
            b"  Dockerfile\x1b[1E",
            b"  LICENSE\x1b[1E",
            b"  README.md\x1b[1E",
            b"  lib/a/b/c/index.js\x1b[1E",
            b"  lib/a/b/c/\xe2\x98\x95.js\x1b[1E",
            b"  lib/a/b/index.js\x1b[1E",
            b"  lib/a/index.js\x1b[1E",
            b"  lib/bar.js\x1b[1E",
            b"\x1b[19d\x1b[1m\x1b[7m.config/bar.toml                                                                                  \x1b[0m\x1b[1E",
            b"\x1b[1m<Up>/<Ctrl-p>:\x1b[0m\x1b[1CUp\x1b[2C\x1b[1m<Down>/<Ctrl-n>:\x1b[0m\x1b[1CDown\x1b[2C\x1b[1m<Enter>:\x1b[0m\x1b[1CExecute\x1b[2C\x1b[1m<C-d>/<C-y>:\x1b[0m\x1b[1CCopy (relative/absolute)",
            b"\x1b8",
            b"\x1b[?1049l"
        )
    );
    let mut ctx = ClipboardContext::new().unwrap();
    if cfg!(windows) {
        assert_eq!(ctx.get_contents().unwrap(), ".config\\bar.toml");
    } else {
        assert_eq!(ctx.get_contents().unwrap(), ".config/bar.toml");
    }
}
