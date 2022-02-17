use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;

use log::{set_boxed_logger, set_max_level, LevelFilter, Log, Metadata, Record};

pub(crate) fn init<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let logger = Logger::new(path)?;
    set_boxed_logger(Box::new(logger)).expect("Unable to set logger");
    set_max_level(LevelFilter::Trace);
    Ok(())
}

#[derive(Debug)]
struct Logger {
    log_file: Mutex<File>,
}

impl Logger {
    fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let log_file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            log_file: Mutex::new(log_file),
        })
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let mut log_file = self
            .log_file
            .lock()
            .expect("Log file writer mutex was poisoned!");
        writeln!(
            log_file,
            "[{}] [{}:{}] {}",
            record.level(),
            record.module_path().unwrap_or_default(),
            record.line().unwrap_or_default(),
            record.args()
        )
        .expect("Unable to write the log file");
    }

    fn flush(&self) {
        let mut log_file = self.log_file.lock().expect("Unable to flush the log file");
        let _ = log_file.flush();
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
        log::info!("test");
        log::debug!("d!");
        assert_eq!(
            &fs::read_to_string(&path).unwrap(),
            "[INFO] [thwack::logger::tests:72] test\n[DEBUG] [thwack::logger::tests:73] d!\n"
        );
    }
}
