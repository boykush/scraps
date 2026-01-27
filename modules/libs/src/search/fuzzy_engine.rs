use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use super::engine::{SearchEngine, SearchLogic};
use super::result::SearchItem;

pub struct FuzzySearchEngine {
    matcher: SkimMatcherV2,
}

impl Default for FuzzySearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzySearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }
}

impl SearchEngine for FuzzySearchEngine {
    fn search(
        &self,
        items: &[SearchItem],
        query: &str,
        num: usize,
        logic: SearchLogic,
    ) -> Vec<SearchItem> {
        if query.is_empty() {
            return items.iter().take(num).cloned().collect();
        }

        // Split query by whitespace
        let keywords: Vec<&str> = query.split_whitespace().collect();

        let mut results_with_scores: Vec<(SearchItem, i64)> = items
            .iter()
            .filter_map(|item| {
                let scores: Vec<i64> = keywords
                    .iter()
                    .filter_map(|kw| self.matcher.fuzzy_match(&item.search_text, kw))
                    .collect();

                match logic {
                    SearchLogic::And => {
                        // AND search: all keywords must match
                        if scores.len() == keywords.len() {
                            Some((item.clone(), scores.iter().sum()))
                        } else {
                            None
                        }
                    }
                    SearchLogic::Or => {
                        // OR search: any keyword can match
                        if !scores.is_empty() {
                            Some((item.clone(), scores.iter().sum()))
                        } else {
                            None
                        }
                    }
                }
            })
            .collect();

        results_with_scores.sort_by(|a, b| b.1.cmp(&a.1));

        results_with_scores
            .into_iter()
            .take(num)
            .map(|(result, _)| result)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn engine() -> FuzzySearchEngine {
        FuzzySearchEngine::new()
    }

    // ===========================================
    // OR logic tests (default)
    // ===========================================

    #[rstest]
    fn test_or_matches_any_keyword(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", ""),
            SearchItem::new("Python Guide", ""),
            SearchItem::new("Rust and Python", ""),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(results.len(), 3);
    }

    #[rstest]
    fn test_or_partial_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Documentation", ""),
            SearchItem::new("Sample", ""),
        ];
        let results = engine.search(&items, "doc", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Documentation");
    }

    #[rstest]
    fn test_or_no_match(engine: FuzzySearchEngine) {
        let items = vec![SearchItem::new("Test", "")];
        let results = engine.search(&items, "xyzzyx", 100, SearchLogic::Or);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_or_searches_body_content(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document A", "contains uniquekeyword here"),
            SearchItem::new("Document B", "no match"),
        ];
        let results = engine.search(&items, "uniquekeyword", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Document A");
    }

    #[rstest]
    fn test_or_empty_query_returns_all(engine: FuzzySearchEngine) {
        let items = vec![SearchItem::new("A", ""), SearchItem::new("B", "")];
        let results = engine.search(&items, "", 100, SearchLogic::Or);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_or_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Item {}", i), "keyword"))
            .collect();
        let results = engine.search(&items, "keyword", 5, SearchLogic::Or);

        assert_eq!(results.len(), 5);
    }

    // ===========================================
    // AND logic tests
    // ===========================================

    #[rstest]
    fn test_and_requires_all_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", ""),
            SearchItem::new("Python Guide", ""),
            SearchItem::new("Rust and Python", ""),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust and Python");
    }

    #[rstest]
    fn test_and_exact_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Test Document", ""),
            SearchItem::new("Another Test", ""),
            SearchItem::new("Document Only", ""),
        ];
        let results = engine.search(&items, "Test Document", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Test Document");
    }

    #[rstest]
    fn test_and_keyword_order_independent(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Programming", ""),
            SearchItem::new("Programming Rust", ""),
            SearchItem::new("Python Language", ""),
        ];
        let results = engine.search(&items, "Programming Rust", 100, SearchLogic::And);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_and_non_consecutive_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust is great for Programming", ""),
            SearchItem::new("Python Language", ""),
        ];
        let results = engine.search(&items, "Rust Programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
    }

    #[rstest]
    fn test_and_no_match_missing_keyword(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", ""),
            SearchItem::new("Python Guide", ""),
        ];
        let results = engine.search(&items, "rust java", 100, SearchLogic::And);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_and_searches_body_content(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document", "rust programming"),
            SearchItem::new("Another", "rust only"),
        ];
        let results = engine.search(&items, "rust programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Document");
    }

    #[rstest]
    fn test_and_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Test Item {}", i), ""))
            .collect();
        let results = engine.search(&items, "Test Item", 5, SearchLogic::And);

        assert_eq!(results.len(), 5);
    }

    // ===========================================
    // Common behavior tests
    // ===========================================

    #[rstest]
    fn test_single_keyword_same_for_or_and_and(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", ""),
            SearchItem::new("Python Guide", ""),
        ];
        let or_results = engine.search(&items, "rust", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "rust", 100, SearchLogic::And);

        assert_eq!(or_results.len(), and_results.len());
    }

    #[rstest]
    fn test_default_logic_is_or(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", ""),
            SearchItem::new("Python Guide", ""),
            SearchItem::new("Rust and Python", ""),
        ];
        let default_results = engine.search(&items, "rust python", 100, SearchLogic::default());
        let or_results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(default_results.len(), or_results.len());
        assert_eq!(default_results.len(), 3);
    }

    #[rstest]
    fn test_empty_items_returns_empty(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = vec![];
        let or_results = engine.search(&items, "test", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "test", 100, SearchLogic::And);

        assert_eq!(or_results.len(), 0);
        assert_eq!(and_results.len(), 0);
    }
}
