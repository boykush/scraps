use super::result::{SearchIndexItem, SearchResult};

pub trait SearchEngine {
    fn search(&self, items: &[SearchIndexItem], query: &str, num: usize) -> Vec<SearchResult>;
}
