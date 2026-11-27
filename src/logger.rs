use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

use log::{LevelFilter, Log, Metadata, Record, set_boxed_logger, set_max_level};

pub(crate) fn init<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let logger = Logger::new(path)?;
    set_boxed_logger(Box::new(logger)).map_err(|e| io::Error::other(e.to_string()))?;
    set_max_level(LevelFilter::Trace);
    Ok(())
}

/// Writes log records to a regular file or a character device.
///
/// `init` installs it as the global logger; tests use `Logger::new` directly.
/// Regular files are created or tightened to owner-only permissions, and each
/// record is escaped so control characters cannot forge lines:
///
/// ```ignore
/// let logger = Logger::new("thwack.log")?;
/// log::set_boxed_logger(Box::new(logger))?;
/// ```
#[derive(Debug)]
struct Logger {
    log_file: Mutex<File>,
}

impl Logger {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let log_file = open_log_file(path.as_ref())?;
        Ok(Self {
            log_file: Mutex::new(log_file),
        })
    }
}

/// Opens the log destination. A regular file is created or tightened to
/// owner-only permissions. Character devices such as `/dev/null` are allowed,
/// and a symlink is followed only when it resolves to a character device (for
/// example `/dev/stderr`), never to a regular file.
fn open_log_file(path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = open_options();
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        match options.open(path) {
            Ok(file) => return prepare_log_file(file),
            Err(e) if e.raw_os_error() == Some(libc::ELOOP) => {
                let mut fallback = open_options();
                fallback.custom_flags(libc::O_NONBLOCK);
                let file = fallback.open(path)?;
                if file.metadata()?.file_type().is_char_device() {
                    return Ok(file);
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "a symlinked log path must resolve to a character device",
                ));
            }
            Err(e) => return Err(e),
        }
    }

    #[cfg(not(unix))]
    {
        prepare_log_file(open_options().open(path)?)
    }
}

fn open_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn prepare_log_file(file: File) -> io::Result<File> {
    let metadata = file.metadata()?;
    let file_type = metadata.file_type();

    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;

        if file_type.is_file() {
            if metadata.uid() != unsafe { libc::geteuid() } {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "the log file must be owned by the current user",
                ));
            }
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
            return Ok(file);
        }
        if file_type.is_char_device() {
            return Ok(file);
        }
    }

    #[cfg(not(unix))]
    {
        if file_type.is_file() {
            return Ok(file);
        }
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "the log file must be a regular file or a character device",
    ))
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let Ok(mut log_file) = self.log_file.lock() else {
            return;
        };
        let message: String = record
            .args()
            .to_string()
            .chars()
            .flat_map(char::escape_debug)
            .collect();
        let _ = writeln!(
            log_file,
            "[{}] [{}:{}] {}",
            record.level(),
            record.module_path().unwrap_or_default(),
            record.line().unwrap_or_default(),
            message
        );
    }

    fn flush(&self) {
        if let Ok(mut log_file) = self.log_file.lock() {
            let _ = log_file.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn info_and_debug() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("log.txt");
        init(&path).unwrap();
        let info_line = line!() + 1;
        log::info!("test");
        let debug_line = line!() + 1;
        log::debug!("d!");
        let contents = &fs::read_to_string(&path).unwrap();
        assert!(contents.contains(&format!("[INFO] [thwack::logger::tests:{info_line}] test")));
        assert!(contents.contains(&format!("[DEBUG] [thwack::logger::tests:{debug_line}] d!")));
    }

    #[cfg(unix)]
    #[test]
    fn new_file_has_owner_only_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir().unwrap();
        let path = tmp.path().join("log.txt");
        let _logger = Logger::new(&path).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o077,
            0,
            "expected owner-only permissions, got {mode:o}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn accepts_a_character_device() {
        assert!(Logger::new("/dev/null").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn tightens_existing_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempdir().unwrap();
        let path = tmp.path().join("log.txt");
        {
            let file = fs::File::create(&path).unwrap();
            file.set_permissions(fs::Permissions::from_mode(0o644))
                .unwrap();
        }

        let _logger = Logger::new(&path).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(
            mode & 0o077,
            0,
            "expected owner-only permissions, got {mode:o}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_follow_a_symlink() {
        use std::os::unix::fs::symlink;

        let tmp = tempdir().unwrap();
        let target = tmp.path().join("target.txt");
        fs::File::create(&target).unwrap();
        let link = tmp.path().join("link.txt");
        symlink(&target, &link).unwrap();

        assert!(Logger::new(&link).is_err());
    }

    #[test]
    fn escapes_control_characters_in_messages() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("log.txt");
        let logger = Logger::new(&path).unwrap();
        let record = Record::builder()
            .args(format_args!("bad\x1b[31m\nname"))
            .level(log::Level::Info)
            .module_path(Some("test"))
            .line(Some(1))
            .build();
        logger.log(&record);

        let contents = fs::read_to_string(&path).unwrap();
        assert!(
            contents.contains("bad\\u{1b}[31m\\nname"),
            "got: {contents:?}"
        );
        assert!(!contents.contains('\x1b'));
    }

    #[test]
    fn does_not_panic_when_the_mutex_is_poisoned() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("log.txt");
        let logger = Logger::new(&path).unwrap();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = logger.log_file.lock().unwrap();
            panic!("poison the mutex");
        }));

        let record = Record::builder()
            .args(format_args!("after poisoning"))
            .level(log::Level::Info)
            .module_path(Some("test"))
            .line(Some(1))
            .build();
        logger.log(&record);
    }
}
