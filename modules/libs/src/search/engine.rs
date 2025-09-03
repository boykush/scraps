use super::result::SearchItem;

pub trait SearchEngine {
    fn search(&self, items: &[SearchItem], query: &str, num: usize) -> Vec<SearchItem>;
}
