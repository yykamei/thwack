use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MatchedPath {
    /// *absolute* is an absolute path/
    absolute: String,

    /// *relative* is a path to `starting_point` passed as an argument of `new`.
    ///
    /// CAUTION: You should not pass it as a target file for command
    ///          because `starting_point` sometimes differs from the current working directory,
    ///          in which this program is run.
    relative: String,

    /// *depth* is the number of path separator.
    depth: usize,

    /// *positions* is a vector containing the matched indicies of *relative*.
    positions: Vec<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Chunk {
    value: String,
    range: Range<usize>,
    matched: bool,
}

impl MatchedPath {
    /// Creates an instance of `MatchedPath`.
    pub(crate) fn new(query: &str, starting_point: &str, absolute: &str) -> Option<Self> {
        let relative = relative(starting_point, absolute);
        let depth = depth_from(relative);
        let positions = positions_from(query, relative)?;
        Some(Self {
            absolute: absolute.to_string(),
            relative: relative.to_string(),
            depth,
            positions,
        })
    }

    /// Returns the absolute path.
    pub(crate) fn absolute(&self) -> &str {
        &self.absolute
    }

    /// Returns the slice of Chunks. This generates reduced chunks if `Self::width` exceeds the `max_width`.
    pub(crate) fn chunks(&self, max_width: usize) -> Vec<Chunk> {
        let width = self.width();
        let mut start = 0;
        if width > max_width {
            let max_width = max_width - 3; // NOTE: `...` requires 3 columns.
            let mut accum = 0;
            for (idx, s) in self.relative.grapheme_indices(true).rev() {
                accum += s.width_cjk();
                if accum > max_width {
                    break;
                }
                start = idx;
            }
        }
        chunks_from(&self.relative[start..], &self.positions[..], start)
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

    /// Returns the width in terminal columns
    fn width(&self) -> usize {
        self.relative.width_cjk()
    }
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

impl Chunk {
    pub(crate) fn matched(&self) -> bool {
        self.matched
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

/// Generates relative from the `starting_point` and `absolute`.
fn relative<'a>(starting_point: &'a str, absolute: &'a str) -> &'a str {
    let relative = absolute
        .strip_prefix(starting_point)
        .expect("The passed starting_point must be prefix of the path.");
    if relative.starts_with(&['/', '\\'][..]) {
        &relative[1..]
    } else {
        relative
    }
}

/// Calculates depth of the `relative` by counting `'/'` or `'\\'`.
fn depth_from(relative: &str) -> usize {
    relative.graphemes(true).fold(
        0,
        |acc, c| {
            if c == "/" || c == "\\" {
                acc + 1
            } else {
                acc
            }
        },
    )
}

// TODO: This should change the algorithm of calculation by `query`.
//       For example, the `query` is `"src/"` (suffixed with `'/'`), a user is expecting items that start with `"src/"`.
/// Calculates matched positions of `relative` with `query`.
/// This searches for characters of `query` in `relative` from the right one by one.
fn positions_from(query: &str, relative: &str) -> Option<Vec<usize>> {
    let mut positions: VecDeque<usize> = VecDeque::with_capacity(query.len());
    for q in normalize_query(query).graphemes(true).rev() {
        let end = if let Some(pos) = positions.front() {
            *pos
        } else {
            relative.len()
        };
        let target = &relative[..end];
        let pos = target
            .grapheme_indices(true)
            .rfind(|(_idx, s)| q.eq_ignore_ascii_case(s))
            .map(|(idx, _)| idx)?;
        positions.push_front(pos);
    }
    Some(positions.into())
}

fn chunks_from(relative: &str, positions: &[usize], offset: usize) -> Vec<Chunk> {
    // NOTE: Allocate more capacity than the actual number of chunks.
    let mut chunks: Vec<Chunk> = Vec::with_capacity(relative.len() / 2);
    if offset > 0 {
        chunks.push(Chunk {
            value: String::from("..."),
            range: (0..3),
            matched: false,
        })
    }
    let mut grapheme_indices = relative.grapheme_indices(true).peekable();
    while let Some((idx, s)) = grapheme_indices.next() {
        let next_idx = grapheme_indices
            .peek()
            .map(|(i, _)| *i)
            .unwrap_or_else(|| relative.len());
        let matched = positions.contains(&(idx + offset));
        match chunks.last_mut() {
            Some(chunk) if chunk.matched == matched => {
                chunk.value.push_str(s);
                chunk.range.end = next_idx;
            }
            Some(_) => chunks.push(Chunk {
                value: s.to_string(),
                matched,
                range: (idx..next_idx),
            }),
            None => chunks.push(Chunk {
                value: s.to_string(),
                matched,
                range: (idx..next_idx),
            }),
        }
    }
    chunks
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

    fn assert_chunks_eq_relative(path: MatchedPath, max_width: usize) {
        let chunks: String = path
            .chunks(max_width)
            .iter()
            .map(|c| c.value.clone())
            .collect::<Vec<String>>()
            .join("");
        assert_eq!(chunks, path.relative)
    }

    #[test]
    fn returns_new_instance() {
        assert_eq!(
            new("abc.txt", "/", "/abc/abc/abc.txt"),
            MatchedPath {
                absolute: String::from("/abc/abc/abc.txt"),
                relative: String::from("abc/abc/abc.txt"),
                positions: vec![8, 9, 10, 11, 12, 13, 14],
                depth: 2,
            },
        );
        assert_eq!(
            new("abc", "/", "/abc/abc/abc.txt"),
            MatchedPath {
                absolute: String::from("/abc/abc/abc.txt"),
                relative: String::from("abc/abc/abc.txt"),
                positions: vec![8, 9, 10],
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
                positions: vec![7, 8, 15],
                depth: 1,
            },
        );
        assert_eq!(
            new("foo☕t", "\\Folder\\", "\\Folder\\foo\\bar\\☕.txt"),
            MatchedPath {
                absolute: String::from("\\Folder\\foo\\bar\\☕.txt"),
                relative: String::from("foo\\bar\\☕.txt"),
                positions: vec![0, 1, 2, 8, 14],
                depth: 2,
            },
        );
        assert_eq!(
            new("a̐éö̲", "/", "/abc/Aa̐Béö̲.txt"),
            MatchedPath {
                absolute: String::from("/abc/Aa̐Béö̲.txt"),
                relative: String::from("abc/Aa̐Béö̲.txt"),
                positions: vec![5, 9, 12],
                depth: 1,
            },
        );
    }

    #[test]
    fn joined_chunks_are_equal_to_relative() {
        assert_chunks_eq_relative(new("abc", "/home", "/home/abc.txt"), 30);
        assert_chunks_eq_relative(new("sbc", "/", "/home/src/abc.txt"), 30);
        assert_chunks_eq_relative(new("☕lover", "/", "/Docs/☕/level/oh/version.txt"), 30);
        assert_chunks_eq_relative(new("passwd", "/etc", "/etc/passwd"), 30);
    }

    #[test]
    fn returns_absolute() {
        let path = new("abc", "/home", "/home/abc.txt");
        assert_eq!(path.absolute(), "/home/abc.txt");
    }

    #[test]
    fn returns_chunks() {
        assert_eq!(
            new("abc.txt", "/", "/abc/abc/abc.txt").chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/abc/"),
                    matched: false,
                    range: (0..8),
                },
                Chunk {
                    value: String::from("abc.txt"),
                    matched: true,
                    range: (8..15),
                },
            ],
        );
        assert_eq!(
            new("abc", "/", "/abc/abc/abc.txt").chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/abc/"),
                    matched: false,
                    range: (0..8),
                },
                Chunk {
                    value: String::from("abc"),
                    matched: true,
                    range: (8..11),
                },
                Chunk {
                    value: String::from(".txt"),
                    matched: false,
                    range: (11..15),
                },
            ],
        );
        assert_eq!(
            new(
                "tem",
                "C:\\Documents",
                "C:\\Documents\\Newsletters\\Summer2018.pdf"
            )
            .chunks(30),
            vec![
                Chunk {
                    value: String::from("Newslet"),
                    matched: false,
                    range: (0..7),
                },
                Chunk {
                    value: String::from("te"),
                    matched: true,
                    range: (7..9),
                },
                Chunk {
                    value: String::from("rs\\Sum"),
                    matched: false,
                    range: (9..15),
                },
                Chunk {
                    value: String::from("m"),
                    matched: true,
                    range: (15..16),
                },
                Chunk {
                    value: String::from("er2018.pdf"),
                    matched: false,
                    range: (16..26),
                },
            ],
        );
        assert_eq!(
            new("foo☕t", "\\Folder\\", "\\Folder\\foo\\bar\\☕.txt").chunks(30),
            vec![
                Chunk {
                    value: String::from("foo"),
                    matched: true,
                    range: (0..3),
                },
                Chunk {
                    value: String::from("\\bar\\"),
                    matched: false,
                    range: (3..8),
                },
                Chunk {
                    value: String::from("☕"),
                    matched: true,
                    range: (8..11),
                },
                Chunk {
                    value: String::from(".tx"),
                    matched: false,
                    range: (11..14),
                },
                Chunk {
                    value: String::from("t"),
                    matched: true,
                    range: (14..15),
                },
            ],
        );
        assert_eq!(
            new("a̐éö̲", "/", "/abc/Aa̐Béö̲.txt").chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/A"),
                    matched: false,
                    range: (0..5),
                },
                Chunk {
                    value: String::from("a̐"),
                    matched: true,
                    range: (5..8),
                },
                Chunk {
                    value: String::from("B"),
                    matched: false,
                    range: (8..9),
                },
                Chunk {
                    value: String::from("éö̲"),
                    matched: true,
                    range: (9..17),
                },
                Chunk {
                    value: String::from(".txt"),
                    matched: false,
                    range: (17..21),
                },
            ],
        );
        assert_eq!(
            new("☕.txt", "/", "/abc/☕/abc/☕.txt").chunks(15),
            vec![
                Chunk {
                    value: String::from(".../abc/"),
                    matched: false,
                    range: (0..5),
                },
                Chunk {
                    value: String::from("☕.txt"),
                    matched: true,
                    range: (5..12),
                },
            ],
        );
        assert_eq!(
            new("👩‍🔬☕", "C:\\", "C:\\Documents\\👩‍🔬\\🦑\\abcde\\☕🌍.txt").chunks(24),
            vec![
                Chunk {
                    value: String::from("...\\🦑\\abcde\\"),
                    matched: false,
                    range: (0..12),
                },
                Chunk {
                    value: String::from("☕"),
                    matched: true,
                    range: (12..15),
                },
                Chunk {
                    value: String::from("🌍.txt"),
                    matched: false,
                    range: (15..23),
                },
            ],
        );
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
