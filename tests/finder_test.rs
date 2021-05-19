use pinpoint::{Finder, MatchedPath, Result};

use crate::helper::create_tree;

mod helper;

#[test]
fn returns_all_paths() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(dir.path().to_str().unwrap(), "").unwrap();
    let size = finder.collect::<Vec<Result<MatchedPath>>>().len();
    assert_eq!(size, 29);
}

#[test]
fn returns_empty() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(
        dir.path().to_str().unwrap(),
        "the word should be not found with 🎂",
    )
    .unwrap();
    let size = finder.collect::<Vec<Result<MatchedPath>>>().len();
    assert_eq!(size, 0);
}

#[test]
fn returns_filtered_paths_with_only_separator() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(dir.path().to_str().unwrap(), "/").unwrap();
    let size = finder.collect::<Vec<Result<MatchedPath>>>().len();
    assert_eq!(size, 15);
}

#[test]
fn returns_filtered_paths_with_uppercase() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(dir.path().to_str().unwrap(), "licenSE").unwrap();
    let mut paths = vec![];
    for matched in finder {
        let path = matched.unwrap();
        paths.push(path);
    }
    let mut paths: Vec<String> = paths
        .iter()
        .map(|m| m.relative.replace('\\', "/"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths, vec!["LICENSE"]);
}

#[test]
fn returns_filtered_paths_with_emoji_coffee() {
    let dir = create_tree().unwrap();
    let finder = Finder::new(dir.path().to_str().unwrap(), "☕").unwrap();
    let mut paths = vec![];
    for matched in finder {
        let path = matched.unwrap();
        paths.push(path);
    }
    let mut paths: Vec<String> = paths
        .iter()
        .map(|m| m.relative.replace('\\', "/"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 3);
    assert_eq!(paths, vec!["lib/a/b/c/☕.js", "src/a/☕.js", "☕.txt"]);
}
