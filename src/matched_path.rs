use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};

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

    /// *absolute_positions* is a vector containing the matched indicies of *absolute*.
    absolute_positions: Vec<usize>,

    /// *relative_positions* is a vector containing the matched indicies of *relative*.
    relative_positions: Vec<usize>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Chunk {
    value: String,
    matched: bool,
}

impl MatchedPath {
    /// Creates an instance of `MatchedPath`.
    pub(crate) fn new(query: &str, starting_point: &str, absolute: &str) -> Option<Self> {
        let relative = relative(starting_point, absolute);
        let depth = depth_from(relative);
        let absolute_positions = positions_from(query, absolute)?;
        let relative_positions = positions_from(query, relative)?;
        Some(Self {
            absolute: absolute.to_string(),
            relative: relative.to_string(),
            depth,
            absolute_positions,
            relative_positions,
        })
    }

    /// Returns the absolute path.
    pub(crate) fn absolute(&self) -> &str {
        &self.absolute
    }

    /// Returns the truncated absolute path.
    pub(crate) fn truncated_absolute(&self, max_width: usize) -> String {
        let chunks = self.absolute_chunks(max_width);
        chunks.iter().map(|c| format!("{}", c)).collect()
    }

    /// Returns the chunks of `absolute`. This generates reduced chunks if the width of the `absolute` exceeds the `max_width`.
    pub(crate) fn absolute_chunks(&self, max_width: usize) -> Vec<Chunk> {
        chunks_from(&self.absolute, &self.absolute_positions[..], max_width)
    }

    /// Returns the chunks of `relative`. This generates reduced chunks if the width of the `absolute` exceeds the `max_width`.
    pub(crate) fn relative_chunks(&self, max_width: usize) -> Vec<Chunk> {
        chunks_from(&self.relative, &self.relative_positions[..], max_width)
    }

    /// Calculate the total distance between each position.
    /// For example, if the `positions` is `vec![1, 2, 3]`, then the distance will be `2`.
    /// If the `positions` is `vec![1, 4, 5]`, then the distance will be `4`.
    /// The least distance is `length of query - 1`.
    fn distance(&self) -> usize {
        let mut iter = self.relative_positions.iter().peekable();
        let mut total = 0;
        while let Some(pos) = iter.next() {
            if let Some(next) = iter.peek() {
                total += *next - pos;
            }
        }
        total
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
/// Calculates matched positions of `path` with `query`.
/// This searches for characters of `query` in `path` from the right one by one.
fn positions_from(query: &str, path: &str) -> Option<Vec<usize>> {
    let mut positions: VecDeque<usize> = VecDeque::with_capacity(query.len());
    for q in normalize_query(query).graphemes(true).rev() {
        let end = if let Some(pos) = positions.front() {
            *pos
        } else {
            path.len()
        };
        let target = &path[..end];
        let pos = target
            .grapheme_indices(true)
            .rfind(|(_idx, s)| q.eq_ignore_ascii_case(s))
            .map(|(idx, _)| idx)?;
        positions.push_front(pos);
    }
    Some(positions.into())
}

/// Returns the width in terminal columns
fn width_of(path: &str) -> usize {
    path.width_cjk()
}

fn chunks_from(path: &str, positions: &[usize], max_width: usize) -> Vec<Chunk> {
    let mut offset = 0;
    if width_of(path) > max_width {
        let max_width = max_width - 3; // NOTE: `...` requires 3 columns.
        let mut accum = 0;
        for (idx, s) in path.grapheme_indices(true).rev() {
            accum += s.width_cjk();
            if accum > max_width {
                break;
            }
            offset = idx;
        }
    }

    // NOTE: Allocate more capacity than the actual number of chunks.
    let mut chunks: Vec<Chunk> = Vec::with_capacity(path.len() / 2);
    if offset > 0 {
        chunks.push(Chunk {
            value: String::from("..."),
            matched: false,
        })
    }
    let mut grapheme_indices = path.grapheme_indices(true);
    while let Some((idx, s)) = grapheme_indices.next() {
        if idx < offset {
            continue;
        }
        let matched = positions.contains(&idx);
        match chunks.last_mut() {
            Some(chunk) if chunk.matched == matched => {
                chunk.value.push_str(s);
            }
            Some(_) => chunks.push(Chunk {
                value: s.to_string(),
                matched,
            }),
            None => chunks.push(Chunk {
                value: s.to_string(),
                matched,
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
            .relative_chunks(max_width)
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
                absolute_positions: vec![9, 10, 11, 12, 13, 14, 15],
                relative_positions: vec![8, 9, 10, 11, 12, 13, 14],
                depth: 2,
            },
        );
        assert_eq!(
            new("abc", "/", "/abc/abc/abc.txt"),
            MatchedPath {
                absolute: String::from("/abc/abc/abc.txt"),
                relative: String::from("abc/abc/abc.txt"),
                absolute_positions: vec![9, 10, 11],
                relative_positions: vec![8, 9, 10],
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
                absolute_positions: vec![20, 21, 28],
                relative_positions: vec![7, 8, 15],
                depth: 1,
            },
        );
        assert_eq!(
            new("foo☕t", "\\Folder\\", "\\Folder\\foo\\bar\\☕.txt"),
            MatchedPath {
                absolute: String::from("\\Folder\\foo\\bar\\☕.txt"),
                relative: String::from("foo\\bar\\☕.txt"),
                absolute_positions: vec![8, 9, 10, 16, 22],
                relative_positions: vec![0, 1, 2, 8, 14],
                depth: 2,
            },
        );
        assert_eq!(
            new("a̐éö̲", "/", "/abc/Aa̐Béö̲.txt"),
            MatchedPath {
                absolute: String::from("/abc/Aa̐Béö̲.txt"),
                relative: String::from("abc/Aa̐Béö̲.txt"),
                absolute_positions: vec![6, 10, 13],
                relative_positions: vec![5, 9, 12],
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
    fn returns_truncated_absolute() {
        let path = new("abc", "/home", "/home/☕/special/test/bar/🚞/abc.txt");
        assert_eq!(
            path.truncated_absolute(100),
            "/home/☕/special/test/bar/🚞/abc.txt"
        );

        let path = new("abc", "/home", "/home/☕/special/test/bar/🚞/abc.txt");
        assert_eq!(path.truncated_absolute(20), "...st/bar/🚞/abc.txt");
    }

    #[test]
    fn returns_absolute_chunks() {
        assert_eq!(
            new("foo.txt", "/", "/foo/abc/foo.txt").absolute_chunks(30),
            vec![
                Chunk {
                    value: String::from("/foo/abc/"),
                    matched: false,
                },
                Chunk {
                    value: String::from("foo.txt"),
                    matched: true,
                },
            ],
        );
        assert_eq!(
            new("abc.txt", "/", "/morning/morning/abc.txt").absolute_chunks(30),
            vec![
                Chunk {
                    value: String::from("/morning/morning/"),
                    matched: false,
                },
                Chunk {
                    value: String::from("abc.txt"),
                    matched: true,
                },
            ],
        );
        assert_eq!(
            new(
                "gs",
                "C:\\Downloads",
                "C:\\Downloads\\Final\\Porting\\Special2019.pdf"
            )
            .absolute_chunks(28),
            vec![
                Chunk {
                    value: String::from("...l\\Portin"),
                    matched: false,
                },
                Chunk {
                    value: String::from("g"),
                    matched: true,
                },
                Chunk {
                    value: String::from("\\"),
                    matched: false,
                },
                Chunk {
                    value: String::from("S"),
                    matched: true,
                },
                Chunk {
                    value: String::from("pecial2019.pdf"),
                    matched: false,
                },
            ],
        );
        assert_eq!(
            new("👩‍🔬🗑", "C:\\", "C:\\Documents\\👩‍🔬\\🦑\\abcde\\🗑🌍.txt").absolute_chunks(24),
            vec![
                Chunk {
                    value: String::from("..."),
                    matched: false
                },
                Chunk {
                    value: String::from("👩‍🔬"),
                    matched: true
                },
                Chunk {
                    value: String::from("\\🦑\\abcde\\"),
                    matched: false
                },
                Chunk {
                    value: String::from("🗑"),
                    matched: true
                },
                Chunk {
                    value: String::from("🌍.txt"),
                    matched: false
                }
            ]
        );
    }

    #[test]
    fn returns_relative_chunks() {
        assert_eq!(
            new("abc.txt", "/", "/abc/abc/abc.txt").relative_chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/abc/"),
                    matched: false,
                },
                Chunk {
                    value: String::from("abc.txt"),
                    matched: true,
                },
            ],
        );
        assert_eq!(
            new("abc", "/", "/abc/abc/abc.txt").relative_chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/abc/"),
                    matched: false,
                },
                Chunk {
                    value: String::from("abc"),
                    matched: true,
                },
                Chunk {
                    value: String::from(".txt"),
                    matched: false,
                },
            ],
        );
        assert_eq!(
            new(
                "tem",
                "C:\\Documents",
                "C:\\Documents\\Newsletters\\Summer2018.pdf"
            )
            .relative_chunks(30),
            vec![
                Chunk {
                    value: String::from("Newslet"),
                    matched: false,
                },
                Chunk {
                    value: String::from("te"),
                    matched: true,
                },
                Chunk {
                    value: String::from("rs\\Sum"),
                    matched: false,
                },
                Chunk {
                    value: String::from("m"),
                    matched: true,
                },
                Chunk {
                    value: String::from("er2018.pdf"),
                    matched: false,
                },
            ],
        );
        assert_eq!(
            new("foo☕t", "\\Folder\\", "\\Folder\\foo\\bar\\☕.txt").relative_chunks(30),
            vec![
                Chunk {
                    value: String::from("foo"),
                    matched: true,
                },
                Chunk {
                    value: String::from("\\bar\\"),
                    matched: false,
                },
                Chunk {
                    value: String::from("☕"),
                    matched: true,
                },
                Chunk {
                    value: String::from(".tx"),
                    matched: false,
                },
                Chunk {
                    value: String::from("t"),
                    matched: true,
                },
            ],
        );
        assert_eq!(
            new("a̐éö̲", "/", "/abc/Aa̐Béö̲.txt").relative_chunks(30),
            vec![
                Chunk {
                    value: String::from("abc/A"),
                    matched: false,
                },
                Chunk {
                    value: String::from("a̐"),
                    matched: true,
                },
                Chunk {
                    value: String::from("B"),
                    matched: false,
                },
                Chunk {
                    value: String::from("éö̲"),
                    matched: true,
                },
                Chunk {
                    value: String::from(".txt"),
                    matched: false,
                },
            ],
        );
        assert_eq!(
            new("☕.txt", "/", "/abc/☕/abc/☕.txt").relative_chunks(15),
            vec![
                Chunk {
                    value: String::from(".../abc/"),
                    matched: false,
                },
                Chunk {
                    value: String::from("☕.txt"),
                    matched: true,
                },
            ],
        );
        assert_eq!(
            new("👩‍🔬☕", "C:\\", "C:\\Documents\\👩‍🔬\\🦑\\abcde\\☕🌍.txt").relative_chunks(24),
            vec![
                Chunk {
                    value: String::from("...\\🦑\\abcde\\"),
                    matched: false,
                },
                Chunk {
                    value: String::from("☕"),
                    matched: true,
                },
                Chunk {
                    value: String::from("🌍.txt"),
                    matched: false,
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
