use std::fmt;

use unicode_segmentation::UnicodeSegmentation;

/// A Query is a container that saves the current query and the cursor position.
#[derive(Debug, Default)]
pub(crate) struct Query {
    value: Vec<String>,
    idx: usize,
}

impl Query {
    pub(crate) fn new(value: &str) -> Self {
        let value: Vec<String> = value
            .graphemes(true)
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let idx = value.len();

        Self { value, idx }
    }

    pub(crate) fn push<S: ToString>(&mut self, s: S) {
        self.value.insert(self.idx, s.to_string());
        self.idx += 1;
    }

    pub(crate) fn pop(&mut self) {
        if self.idx > 0 {
            self.value.remove(self.idx - 1);
            self.idx -= 1;
        }
    }

    pub(crate) fn move_left(&mut self) {
        if self.idx > 0 {
            self.idx -= 1;
        }
    }

    pub(crate) fn move_right(&mut self) {
        if self.idx < self.value.len() {
            self.idx += 1;
        }
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.value.join("").as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let mut query = Query::new("👩‍🔬!");
        assert_eq!(query.value, vec!["👩‍🔬", "!"]);
        assert_eq!(query.idx, 2);

        query.push("💇‍♂️");
        query.push("a");
        query.push("b");
        assert_eq!(query.value, vec!["👩‍🔬", "!", "💇‍♂️", "a", "b"]);
        assert_eq!(query.idx, 5);

        query.pop();
        assert_eq!(query.value, vec!["👩‍🔬", "!", "💇‍♂️", "a"]);
        assert_eq!(query.idx, 4);

        query.move_left();
        assert_eq!(query.idx, 3);

        query.pop();
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a"]);
        assert_eq!(query.idx, 2);

        query.move_right();
        query.move_right();
        query.move_right();
        assert_eq!(query.idx, 3);

        query.push("?");
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 4);

        query.move_left();
        query.move_left();
        query.move_left();
        query.move_left();
        query.move_left();
        query.move_left();
        assert_eq!(query.idx, 0);

        query.pop();
        assert_eq!(query.value, vec!["👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 0);

        query.push("😇");
        assert_eq!(query.value, vec!["😇", "👩‍🔬", "!", "a", "?"]);
        assert_eq!(query.idx, 1);
    }

    #[test]
    fn test_to_string() {
        let query = Query::new("Super cool query");
        assert_eq!(
            query.value,
            vec!["S", "u", "p", "e", "r", " ", "c", "o", "o", "l", " ", "q", "u", "e", "r", "y"]
        );
        assert_eq!(query.idx, 16);
        assert_eq!(query.to_string(), "Super cool query");
    }
}
