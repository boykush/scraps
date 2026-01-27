use super::result::SearchItem;

/// Search logic for combining multiple keywords
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchLogic {
    /// All keywords must match
    And,
    /// Any keyword can match (default)
    #[default]
    Or,
}

pub trait SearchEngine {
    fn search(
        &self,
        items: &[SearchItem],
        query: &str,
        num: usize,
        logic: SearchLogic,
    ) -> Vec<SearchItem>;
}
