use pinpoint::{Finder, MatchedPath, Result};

use crate::helper::create_tree;

mod helper;

#[test]
fn returns_all_paths() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(dir.path(), "").unwrap();
    let size = finder.collect::<Vec<Result<MatchedPath>>>().len();
    assert_eq!(size, 27);
}

// TODO: Test more
