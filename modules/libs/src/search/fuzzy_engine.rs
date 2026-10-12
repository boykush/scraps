use std::cmp::Reverse;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::nfc::nfc;

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

        let query = nfc(query);
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

        results_with_scores.sort_by_key(|(_, score)| Reverse(*score));

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
            SearchItem::new("Rust Guide", None, ""),
            SearchItem::new("Python Guide", None, ""),
            SearchItem::new("Rust and Python", None, ""),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(results.len(), 3);
    }

    #[rstest]
    fn test_or_partial_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Documentation", None, ""),
            SearchItem::new("Sample", None, ""),
        ];
        let results = engine.search(&items, "doc", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Documentation");
    }

    #[rstest]
    fn test_or_no_match(engine: FuzzySearchEngine) {
        let items = vec![SearchItem::new("Test", None, "")];
        let results = engine.search(&items, "xyzzyx", 100, SearchLogic::Or);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_or_searches_body_content(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document A", None, "contains uniquekeyword here"),
            SearchItem::new("Document B", None, "no match"),
        ];
        let results = engine.search(&items, "uniquekeyword", 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Document A");
    }

    #[rstest]
    fn test_or_empty_query_returns_all(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("A", None, ""),
            SearchItem::new("B", None, ""),
        ];
        let results = engine.search(&items, "", 100, SearchLogic::Or);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_or_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Item {}", i), None, "keyword"))
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
            SearchItem::new("Rust Guide", None, ""),
            SearchItem::new("Python Guide", None, ""),
            SearchItem::new("Rust and Python", None, ""),
        ];
        let results = engine.search(&items, "rust python", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Rust and Python");
    }

    #[rstest]
    fn test_and_exact_match(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Test Document", None, ""),
            SearchItem::new("Another Test", None, ""),
            SearchItem::new("Document Only", None, ""),
        ];
        let results = engine.search(&items, "Test Document", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Test Document");
    }

    #[rstest]
    #[case::composed_query("caf\u{e9}", "cafe\u{301} menu")]
    #[case::decomposed_query("cafe\u{301}", "caf\u{e9} menu")]
    fn test_matches_across_normalization_forms(
        engine: FuzzySearchEngine,
        #[case] query: &str,
        #[case] body: &str,
    ) {
        let items = vec![
            SearchItem::new("note", None, body),
            SearchItem::new("other", None, ""),
        ];
        let results = engine.search(&items, query, 100, SearchLogic::Or);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "note");
    }

    #[rstest]
    fn test_and_keyword_order_independent(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Programming", None, ""),
            SearchItem::new("Programming Rust", None, ""),
            SearchItem::new("Python Language", None, ""),
        ];
        let results = engine.search(&items, "Programming Rust", 100, SearchLogic::And);

        assert_eq!(results.len(), 2);
    }

    #[rstest]
    fn test_and_non_consecutive_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust is great for Programming", None, ""),
            SearchItem::new("Python Language", None, ""),
        ];
        let results = engine.search(&items, "Rust Programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
    }

    #[rstest]
    fn test_and_no_match_missing_keyword(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, ""),
            SearchItem::new("Python Guide", None, ""),
        ];
        let results = engine.search(&items, "rust java", 100, SearchLogic::And);

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_and_searches_body_content(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document", None, "rust programming"),
            SearchItem::new("Another", None, "rust only"),
        ];
        let results = engine.search(&items, "rust programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "Document");
    }

    #[rstest]
    fn test_and_result_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Test Item {}", i), None, ""))
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
            SearchItem::new("Rust Guide", None, ""),
            SearchItem::new("Python Guide", None, ""),
        ];
        let or_results = engine.search(&items, "rust", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "rust", 100, SearchLogic::And);

        assert_eq!(or_results.len(), and_results.len());
    }

    #[rstest]
    fn test_default_logic_is_or(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Guide", None, ""),
            SearchItem::new("Python Guide", None, ""),
            SearchItem::new("Rust and Python", None, ""),
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
