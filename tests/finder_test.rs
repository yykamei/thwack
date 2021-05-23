use thwack::Finder;

use crate::helper::create_tree;

mod helper;

fn find_paths(starting_point: &str, query: &str) -> Vec<String> {
    let mut paths = vec![];
    for matched in Finder::new(starting_point, query).unwrap() {
        let path = matched.unwrap();
        paths.push(path);
    }
    let mut paths: Vec<String> = paths
        .iter()
        .map(|m| m.relative.replace('\\', "/"))
        .collect();
    paths.sort();
    paths
}

#[test]
fn returns_all_paths() {
    let dir = create_tree().unwrap();
    let size = find_paths(dir.path().to_str().unwrap(), "").len();
    assert_eq!(size, 29);
}

#[test]
fn returns_empty() {
    let dir = create_tree().unwrap();
    let size = find_paths(
        dir.path().to_str().unwrap(),
        "the word should be not found with 🎂",
    )
    .len();
    assert_eq!(size, 0);
}

#[test]
fn returns_filtered_paths_with_only_separator() {
    let dir = create_tree().unwrap();
    let size = find_paths(dir.path().to_str().unwrap(), "/").len();
    assert_eq!(size, 15);
}

#[test]
fn returns_filtered_paths_with_uppercase() {
    let dir = create_tree().unwrap();
    let paths = find_paths(dir.path().to_str().unwrap(), "licenSE");
    assert_eq!(paths.len(), 1);
    assert_eq!(paths, vec!["LICENSE"]);
}

#[test]
fn returns_filtered_paths_with_emoji_coffee() {
    let dir = create_tree().unwrap();
    let paths = find_paths(dir.path().to_str().unwrap(), "☕");
    assert_eq!(paths.len(), 3);
    assert_eq!(paths, vec!["lib/a/b/c/☕.js", "src/a/☕.js", "☕.txt"]);
}
