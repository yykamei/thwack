use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MatchedPath {
    absolute: String,
    relative: String,
    positions: VecDeque<usize>,
    depth: usize,
}

impl MatchedPath {
    /// Creates an instance of `MatchedPath`.
    pub(crate) fn new(query: &str, starting_point: &str, absolute: &str) -> Option<Self> {
        let relative = relative_chars(starting_point, absolute);
        let depth = depth_from(relative.iter());
        let positions = positions_from(query, &relative[..])?;
        Some(Self {
            absolute: absolute.to_string(),
            relative: relative.iter().collect(),
            positions,
            depth,
        })
    }

    /// Returns the relative path
    pub(crate) fn relative(&self) -> &str {
        &self.relative
    }

    /// Calculate the total distance between each position.
    /// For example, if the `positions` is `vec![1, 2, 3]`, then the distance will be `2`.
    /// If the `positions` is `vec![1, 4, 5]`, then the distance will be `4`.
    /// The least distance is `length of query - 1`.
    fn distance(&self) -> usize {
        let mut iter = self.positions.iter().peekable();
        let mut total = 0;
        while let Some(pos) = iter.next() {
            if let Some(next) = iter.peek() {
                total += *next - pos;
            }
        }
        total
    }
    // TODO: Present with colorized value with emphasized positions.
}

impl Display for MatchedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.relative, f)
    }
}

impl Ord for MatchedPath {
    fn cmp(&self, other: &Self) -> Ordering {
        let depth = match self.distance().cmp(&other.distance()) {
            Ordering::Equal => self.depth.cmp(&other.depth),
            any => any,
        };
        match depth {
            Ordering::Equal => self.relative.cmp(&other.relative),
            any => any,
        }
    }
}

impl PartialOrd for MatchedPath {
    fn partial_cmp(&self, other: &MatchedPath) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Generates relative from the `starting_point` and `absolute`.
/// This returns `Vec<char>` instead of other types, such as `String` in order to make it useful for manipulating later.
fn relative_chars(starting_point: &str, absolute: &str) -> Vec<char> {
    let mut relative: VecDeque<char> = absolute
        .strip_prefix(starting_point)
        .expect("The passed starting_point must be prefix of the path.")
        .chars()
        .collect();
    match relative.get(0) {
        Some(c) if *c == '/' || *c == '\\' => {
            relative.pop_front();
        }
        _ => (),
    };
    relative.into()
}

/// Calculates depth of the `relative` by counting `'/'` or `'\\'`.
fn depth_from<'a>(relative: impl Iterator<Item = &'a char>) -> usize {
    relative.fold(0, |acc, c| {
        if *c == '/' || *c == '\\' {
            acc + 1
        } else {
            acc
        }
    })
}

/// Calculates matched positions of `relative` with `query`.
/// This searches for characters of `query` in `relative` from the right one by one.
fn positions_from(query: &str, relative: &[char]) -> Option<VecDeque<usize>> {
    let mut positions: VecDeque<usize> = VecDeque::with_capacity(query.len());
    for char in normalize_query(query).chars().rev() {
        let end = if let Some(pos) = positions.front() {
            *pos
        } else {
            relative.len()
        };
        let target = &relative[..end];
        let pos = target.iter().rposition(|t| char.eq_ignore_ascii_case(t))?;
        positions.push_front(pos);
    }
    Some(positions)
}

#[cfg(target_os = "windows")]
fn normalize_query(query: &str) -> String {
    // NOTE: Forward slashes are not allowed in a filename, so this replacing is supposed to work.
    //       See https://docs.microsoft.com/en-us/windows/win32/fileio/naming-a-file#naming-conventions
    query.replace('/', "\\")
}

#[cfg(not(target_os = "windows"))]
fn normalize_query(query: &str) -> &str {
    query
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new(query: &str, starting_point: &str, absolute: &str) -> MatchedPath {
        MatchedPath::new(query, starting_point, absolute).unwrap()
    }

    #[test]
    fn returns_new_instance() {
        assert_eq!(
            new("abc.txt", "/", "/abc/abc/abc.txt"),
            MatchedPath {
                absolute: String::from("/abc/abc/abc.txt"),
                relative: String::from("abc/abc/abc.txt"),
                positions: VecDeque::from(vec![8, 9, 10, 11, 12, 13, 14]),
                depth: 2,
            },
        );
        assert_eq!(
            new("abc", "/", "/abc/abc/abc.txt"),
            MatchedPath {
                absolute: String::from("/abc/abc/abc.txt"),
                relative: String::from("abc/abc/abc.txt"),
                positions: VecDeque::from(vec![8, 9, 10]),
                depth: 2,
            },
        );
        assert_eq!(
            new(
                "tem",
                "C:\\Documents",
                "C:\\Documents\\Newsletters\\Summer2018.pdf"
            ),
            MatchedPath {
                absolute: String::from("C:\\Documents\\Newsletters\\Summer2018.pdf"),
                relative: String::from("Newsletters\\Summer2018.pdf"),
                positions: VecDeque::from(vec![7, 8, 15]),
                depth: 1,
            },
        );
        assert_eq!(
            new("foo☕t", "\\Folder\\", "\\Folder\\foo\\bar\\☕.txt"),
            MatchedPath {
                absolute: String::from("\\Folder\\foo\\bar\\☕.txt"),
                relative: String::from("foo\\bar\\☕.txt"),
                positions: VecDeque::from(vec![0, 1, 2, 8, 12]),
                depth: 2,
            },
        );
    }

    #[test]
    fn returns_fields() {
        let path = new("abc", "/home", "/home/abc.txt");
        assert_eq!(path.relative(), "abc.txt");
    }

    #[test]
    fn distance() {
        assert_eq!(new("abc", "/home", "/home/abc.txt").distance(), 2);
        assert_eq!(new("abc", "/home", "/home/a123bc.txt").distance(), 5);
        assert_eq!(new("foo.txt", "/home", "/home/ok/foo.txt").distance(), 6);
        assert_eq!(
            new("foo.txt", "/home", "/home/ok/f1o1o/ok.txt").distance(),
            11
        );
        assert_eq!(new("foo.txt", "/home", "/home/ok/foo/ok.txt").distance(), 9);
    }

    #[test]
    fn sort() {
        let mut given = vec![
            new("abc.txt", "/home", "/home/abc.txt"),
            new("abc.txt", "/home", "/home/a12bc.txt"),
            new("abc.txt", "/home", "/home/a123bc.txt"),
            new("abc.txt", "/home", "/home/abc/cat.txt"),
            new("abc.txt", "/home", "/home/abc/src/abc.txt"),
            new("abc.txt", "/home", "/home/src/abc.txt"),
            new("abc.txt", "/home", "/home/src/n1/n2/aXbc.txt"),
            new("abc.txt", "/home", "/home/src/n1/n2/Foo-aXbc.txt"),
            new("abc.txt", "/home", "/home/src/n1/n2/Foo-aXbXc.txt"),
            new("abc.txt", "/home", "/home/src/n1/n2/abc.txt"),
            new("abc.txt", "/home", "/home/lib/abc!.txt"),
        ];
        given.sort();
        assert_eq!(
            given,
            vec![
                new("abc.txt", "/home", "/home/abc.txt"),
                new("abc.txt", "/home", "/home/src/abc.txt"),
                new("abc.txt", "/home", "/home/abc/src/abc.txt"),
                new("abc.txt", "/home", "/home/src/n1/n2/abc.txt"),
                new("abc.txt", "/home", "/home/lib/abc!.txt"),
                new("abc.txt", "/home", "/home/src/n1/n2/Foo-aXbc.txt"),
                new("abc.txt", "/home", "/home/src/n1/n2/aXbc.txt"),
                new("abc.txt", "/home", "/home/a12bc.txt"),
                new("abc.txt", "/home", "/home/src/n1/n2/Foo-aXbXc.txt"),
                new("abc.txt", "/home", "/home/a123bc.txt"),
                new("abc.txt", "/home", "/home/abc/cat.txt"),
            ],
        );
    }
}
