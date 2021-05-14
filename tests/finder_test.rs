use std::env::current_dir;
use std::path::PathBuf;

use pinpoint::{Finder, Result};

use crate::helper::create_tree;

mod helper;

#[test]
fn returns_all_paths() {
    let dir = create_tree().unwrap();
    let mut finder = Finder::new(dir.path(), "").unwrap();
    let size = finder.collect::<Vec<Result<PathBuf>>>().len();
    assert_eq!(size, 27);
}
