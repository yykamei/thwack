use std::env::ArgsOs;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::io::Write;
use std::process::exit;

use crate::error::{Error, Result};
use crate::finder::Finder;

pub fn entrypoint(args: &mut ArgsOs) -> Result<()> {
    let args = Args::parse(args)?;
    let starting_point = args.starting_point.as_deref().unwrap_or(".");
    let query = args.query.as_deref().unwrap_or("");
    for path in Finder::new(starting_point, query)? {
        let path = path?;
        println!("{}", path); // FIXME: Implement more appropriately, plus sorting is required.
    }
    Ok(())
}

pub fn safe_exit(code: i32) {
    let _ = std::io::stdout().lock().flush();
    let _ = std::io::stderr().lock().flush();
    exit(code)
}

pub struct Args {
    program: String,
    starting_point: Option<String>,
    query: Option<String>,
}

// TODO: Config might be required: e.g. impl From<Args> for Config.
// TODO: Test for Args
impl Args {
    pub(super) fn parse(args: &mut ArgsOs) -> Result<Self> {
        let program = os_str_to_str(
            args.next()
                .expect("The first argument is supposed to be a program name."),
        )?;
        let mut starting_point = None;
        let mut query = None;

        while let Some(arg) = args.next() {
            let arg = os_str_to_str(arg)?;
            match arg.as_ref() {
                // TODO: --help
                "--help" => (),
                x => match (starting_point.as_deref(), query.as_deref()) {
                    (None, None) => query = Some(x.to_string()),
                    (None, Some(q)) => {
                        starting_point = Some(q.to_string());
                        query = Some(x.to_string());
                    }
                    (Some(_), None) => panic!("This pattern should not exist."),
                    (Some(_), Some(_)) => {
                        return Err(Error::args(&format!("Illegal argument: `{}`", x)))
                    }
                },
            }
        }

        Ok(Self {
            program,
            starting_point,
            query,
        })
    }
}

#[inline]
fn os_str_to_str<T: AsRef<OsStr> + Debug>(os_str: T) -> Result<String> {
    os_str
        .as_ref()
        .to_str()
        .ok_or(Error::args(&format!(
            "The specified string `{:?}` does not seem to be valid unicode.",
            os_str
        )))
        .map(|s| s.to_string())
}
