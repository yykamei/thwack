use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub(crate) struct StartingPoint(String);

impl StartingPoint {
    pub(crate) fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = canonicalize_starting_point(dir.as_ref())?;
        let root = path_to_string(&dir)?;
        Ok(Self(root))
    }
}

impl AsRef<str> for StartingPoint {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn canonicalize_starting_point(path: &Path) -> Result<PathBuf> {
    path.canonicalize().map_err(|_e|
        Error::args(&format!(
            "The specified starting point {:?} cannot be normalized. Perhaps, it might not exist or cannot be read.",
            path,
        ))
    )
}

fn path_to_string(path: &Path) -> Result<String> {
    path.to_str().map(|s| s.to_string()).ok_or_else(|| {
        Error::invalid_unicode(&format!(
            "The path {:?} does not seem to be valid unicode.",
            path
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `tmp` is located in the root directory of this repository.
    fn tmp() -> String {
        let path: &Path = "tmp".as_ref();
        path.canonicalize().unwrap().to_str().unwrap().to_string()
    }

    #[test]
    fn new() {
        assert_eq!(StartingPoint::new("tmp").unwrap(), StartingPoint(tmp()));
    }

    #[test]
    fn new_fail_with_non_existent_dir() {
        let result = StartingPoint::new("non_existent_dir");
        assert!(result.is_err());
        assert_eq!(format!("{}", result.unwrap_err()), "The specified starting point \"non_existent_dir\" cannot be normalized. Perhaps, it might not exist or cannot be read.");
    }

    #[test]
    fn as_ref() {
        assert_eq!(StartingPoint::new("tmp").unwrap().as_ref(), &tmp());
    }
}
