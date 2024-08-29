use crate::matched_path::MatchedPath;
use crate::query::Query;
use crate::starting_point::StartingPoint;
use crate::tree::Tree;
use crate::Result;

#[derive(Debug)]
pub(crate) struct Candidates {
    paths: Vec<MatchedPath>,
    selected: Option<usize>,
}

impl Candidates {
    pub(crate) fn new(
        visible_paths_length: usize,
        starting_point: &StartingPoint,
        tree: &Tree,
        query: &Query,
    ) -> Result<Self> {
        let mut paths: Vec<MatchedPath> = Vec::new();
        for path in tree.iter() {
            match MatchedPath::new(&query.to_string(), starting_point.as_ref(), path) {
                Some(matched) => paths.push(matched),
                None => continue,
            }
        }
        paths.sort();
        paths.truncate(visible_paths_length);
        let selected = if paths.is_empty() { None } else { Some(0) };

        Ok(Self { paths, selected })
    }

    pub(crate) fn paths(&self) -> &[MatchedPath] {
        &self.paths
    }

    pub(crate) fn selected(&self) -> Option<&MatchedPath> {
        if let Some(s) = self.selected {
            return self.paths.get(s);
        }
        None
    }

    pub(crate) fn move_down(&mut self) {
        let limit = self.paths.len();
        if limit == 0 {
            return;
        }
        if let Some(s) = self.selected {
            if s < limit - 1 {
                self.selected = Some(s + 1);
            }
        } else {
            self.selected = Some(0);
        }
    }

    pub(crate) fn move_up(&mut self) {
        let limit = self.paths.len();
        if limit == 0 {
            return;
        }
        if let Some(s) = self.selected {
            if s > 0 {
                self.selected = Some(s - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::tests::create_files;
    use git2::Repository;

    #[test]
    fn test_candidates_without_query() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();

        let candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert_eq!(result, &[".browserslistrc", ".editorconfig", ".env"]);
    }

    #[test]
    fn test_candidates_with_query() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("bar");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();
        let candidates = Candidates::new(5, &starting_point, &tree, &query).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert_eq!(result, &[".config/bar.toml", "lib/bar.js"]);
    }

    #[test]
    fn test_candidates_without_repo() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let tree = Tree::new(starting_point.as_ref(), None).unwrap();

        let candidates = Candidates::new(100, &starting_point, &tree, &query).unwrap();
        let result: Vec<String> = candidates
            .paths
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert!(result.contains(&"log.txt".to_string()));
        assert!(result.contains(&".git/config".to_string()));
    }

    #[test]
    fn test_move_down() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();

        let mut candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(1));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));

        let mut candidates = Candidates::new(0, &starting_point, &tree, &query).unwrap();
        candidates.move_down();
        assert_eq!(candidates.selected, None);
        candidates.move_down();
        assert_eq!(candidates.selected, None);
    }

    #[test]
    fn test_move_up() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();

        let mut candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_down();
        candidates.move_down();
        candidates.move_down();
        assert_eq!(candidates.selected, Some(2));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(1));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(0));
        candidates.move_up();
        assert_eq!(candidates.selected, Some(0));

        let mut candidates = Candidates::new(0, &starting_point, &tree, &query).unwrap();
        candidates.move_up();
        assert_eq!(candidates.selected, None);
        candidates.move_up();
        assert_eq!(candidates.selected, None);
    }

    #[test]
    fn test_selected() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();

        let mut candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        assert_eq!(candidates.selected().unwrap().relative(), ".browserslistrc");

        candidates.move_down();
        assert!(candidates
            .selected()
            .unwrap()
            .relative()
            .ends_with(".editorconfig"));

        candidates.move_down();
        assert!(candidates.selected().unwrap().relative().ends_with(".env"));

        candidates.move_up();
        candidates.move_up();
        assert_eq!(candidates.selected().unwrap().relative(), ".browserslistrc");
    }

    #[test]
    fn test_selected_none_at_started() {
        let dir = create_files(false).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("ABCABC!!!!!!!!!");
        let tree = Tree::new(starting_point.as_ref(), None).unwrap();

        let mut candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        assert_eq!(candidates.selected(), None);

        candidates.move_down();
        assert_eq!(candidates.selected(), None);

        candidates.move_down();
    }

    #[test]
    fn test_paths() {
        let dir = create_files(true).unwrap();
        let starting_point = StartingPoint::new(dir.path()).unwrap();
        let query = Query::new("");
        let repo = Repository::open(dir.path()).unwrap();
        let tree = Tree::new(starting_point.as_ref(), Some(&repo)).unwrap();

        let candidates = Candidates::new(3, &starting_point, &tree, &query).unwrap();
        let result: Vec<String> = candidates
            .paths()
            .iter()
            .map(|p| p.relative())
            .map(|m| format!("{}", m).replace('\\', "/"))
            .collect();
        assert_eq!(result, &[".browserslistrc", ".editorconfig", ".env"]);
    }
}
