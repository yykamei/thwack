use std::fmt;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// A Query is a container that saves the current query and the cursor position.
#[derive(Debug, Default)]
pub(crate) struct Query {
    value: Vec<String>,
    idx: usize,
    pub(crate) terminal_pos: usize,
}

impl Query {
    pub(crate) fn new(value: &str) -> Self {
        let value: Vec<String> = value
            .graphemes(true)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let idx = value.len();
        let terminal_pos = value.iter().map(|s| get_cjk_width(s)).sum();

        Self {
            value,
            idx,
            terminal_pos,
        }
    }

    pub(crate) fn push<S: ToString>(&mut self, s: S) -> usize {
        self.value.insert(self.idx, s.to_string());
        self.idx += 1;
        let d = get_cjk_width(&s.to_string());
        self.terminal_pos += d;
        d
    }

    pub(crate) fn pop(&mut self) -> usize {
        if self.idx > 0 {
            let popped = self.value.remove(self.idx - 1);
            self.idx -= 1;
            let d = get_cjk_width(&popped);
            self.terminal_pos -= d;
            return d;
        }
        0
    }

    pub(crate) fn move_left(&mut self) -> usize {
        if self.idx > 0 {
            let char_before_move = self
                .value
                .get(self.idx - 1)
                .expect("Unexpected out of bounds");
            self.idx -= 1;
            let d = get_cjk_width(char_before_move);
            self.terminal_pos -= d;
            return d;
        }
        0
    }

    pub(crate) fn move_right(&mut self) -> usize {
        if self.idx < self.value.len() {
            self.idx += 1;
            let char_after_move = self
                .value
                .get(self.idx - 1)
                .expect("Unexpected out of bounds");
            let d = get_cjk_width(char_after_move);
            self.terminal_pos += d;
            return d;
        }
        0
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.value.join("").as_str(), f)
    }
}

fn get_cjk_width(s: &str) -> usize {
    return if s.width_cjk() == 1 {
        1
    } else {
        2 // 2 is the width of a CJK character. Some unicode emojis have more than 2 width, but most terminal handle such characters as 2 width.
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut query = Query::new("👩‍🔬!");
        assert_eq!(query.value, vec!["👩‍🔬", "!"]);
        assert_eq!(query.idx, 2);
        assert_eq!(query.terminal_pos, 3);

        assert_eq!(query.push("💇‍♂️"), 2);
        assert_eq!(query.push("a"), 1);
        assert_eq!(query.push("b"), 1);
        assert_eq!(query.value, vec!["👩‍🔬", "!", "💇‍♂️", "a", "b"]);
        assert_eq!(query.idx, 5);
        assert_eq!(query.terminal_pos, 7);

        assert_eq!(query.pop(), 1);
        assert_eq!(query.value, vec!["👩‍🔬", "!", "💇‍♂️", "a"]);
        assert_eq!(query.idx, 4);
        assert_eq!(query.terminal_pos, 6);

        assert_eq!(query.move_left(), 1);
        assert_eq!(query.idx, 3);
        assert_eq!(query.terminal_pos, 5);

        assert_eq!(query.pop(), 2);
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a"]);
        assert_eq!(query.idx, 2);
        assert_eq!(query.terminal_pos, 3);

        assert_eq!(query.move_left(), 1);
        assert_eq!(query.move_left(), 2);
        assert_eq!(query.move_right(), 2);
        assert_eq!(query.move_right(), 1);
        assert_eq!(query.move_right(), 1);
        assert_eq!(query.move_right(), 0);
        assert_eq!(query.move_right(), 0);
        assert_eq!(query.idx, 3);
        assert_eq!(query.terminal_pos, 4);

        assert_eq!(query.push("?"), 1);
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 4);
        assert_eq!(query.terminal_pos, 5);

        assert_eq!(query.move_left(), 1);
        assert_eq!(query.move_left(), 1);
        assert_eq!(query.move_left(), 1);
        assert_eq!(query.move_left(), 2);
        assert_eq!(query.move_left(), 0);
        assert_eq!(query.move_left(), 0);
        assert_eq!(query.idx, 0);
        assert_eq!(query.terminal_pos, 0);

        assert_eq!(query.pop(), 0);
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 0);
        assert_eq!(query.terminal_pos, 0);

        assert_eq!(query.push("😇"), 2);
        assert_eq!(query.value, vec!["😇", "👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 1);
        assert_eq!(query.terminal_pos, 2);
    }

    #[test]
    fn test_to_string() {
        let query = Query::new("Super cool query");
        assert_eq!(
            query.value,
            vec![
                "S", "u", "p", "e", "r", " ", "c", "o", "o", "l", " ", "q", "u", "e", "r", "y"
            ]
        );
        assert_eq!(query.idx, 16);
        assert_eq!(query.terminal_pos, 16);
        assert_eq!(query.to_string(), "Super cool query");
    }
}
