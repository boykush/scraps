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

    pub fn with_case_sensitive(case_sensitive: bool) -> Self {
        if case_sensitive {
            Self {
                matcher: SkimMatcherV2::default(),
            }
        } else {
            Self {
                matcher: SkimMatcherV2::default().ignore_case(),
            }
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

    #[fixture]
    fn search_items() -> Vec<SearchItem> {
        vec![
            SearchItem::new("Test Document", ""),
            SearchItem::new("Another Document", ""),
            SearchItem::new("Sample Test", ""),
            SearchItem::new("Documentation", ""),
            SearchItem::new("Testing Framework", ""),
            SearchItem::new("Test Suite", ""),
        ]
    }

    /// Fixture for AND/OR logic comparison tests
    #[fixture]
    fn logic_test_items() -> Vec<SearchItem> {
        vec![
            SearchItem::new("Rust Documentation", "Rust content"),
            SearchItem::new("Python Documentation", "Python content"),
            SearchItem::new("Rust and Python", "Both languages"),
        ]
    }

    // ===========================================
    // Basic fuzzy search tests (using default OR)
    // ===========================================

    #[rstest]
    fn test_fuzzy_search_basic(engine: FuzzySearchEngine, search_items: Vec<SearchItem>) {
        let results = engine.search(&search_items, "test", 100, SearchLogic::default());

        assert!(!results.is_empty());
        assert!(results.len() <= search_items.len());
    }

    #[rstest]
    fn test_fuzzy_search_exact_match(engine: FuzzySearchEngine, search_items: Vec<SearchItem>) {
        let results = engine.search(&search_items, "Test Document", 100, SearchLogic::default());

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Test Document");
    }

    #[rstest]
    fn test_fuzzy_search_partial_match(engine: FuzzySearchEngine, search_items: Vec<SearchItem>) {
        let results = engine.search(&search_items, "doc", 100, SearchLogic::default());
        assert!(!results.is_empty());

        let results = engine.search(&search_items, "fram", 100, SearchLogic::default());
        assert!(!results.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_no_match(engine: FuzzySearchEngine, search_items: Vec<SearchItem>) {
        let results = engine.search(&search_items, "xyzzyx", 100, SearchLogic::default());
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_fuzzy_search_empty_query(engine: FuzzySearchEngine, search_items: Vec<SearchItem>) {
        let results = engine.search(&search_items, "", 100, SearchLogic::default());
        assert_eq!(results.len(), search_items.len());
    }

    #[rstest]
    fn test_fuzzy_search_empty_items(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = vec![];
        let results = engine.search(&items, "test", 100, SearchLogic::default());
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_fuzzy_search_case_insensitive(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::with_case_sensitive(false);

        let results1 = engine.search(&search_items, "test", 100, SearchLogic::default());
        let results2 = engine.search(&search_items, "Test", 100, SearchLogic::default());
        let results3 = engine.search(&search_items, "TEST", 100, SearchLogic::default());

        assert!(!results1.is_empty());
        assert_eq!(results1.len(), results2.len());
        assert_eq!(results2.len(), results3.len());
    }

    #[rstest]
    fn test_fuzzy_search_case_sensitive(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::with_case_sensitive(true);

        // "Test" matches items with "Test" (case-sensitive)
        let results = engine.search(&search_items, "Test", 100, SearchLogic::default());
        assert!(!results.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_body_content(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Document A", "contains uniquekeyword here"),
            SearchItem::new("Document B", "no match"),
        ];

        let results = engine.search(&items, "uniquekeyword", 100, SearchLogic::default());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Document A");
    }

    // ===========================================
    // Result limit tests
    // ===========================================

    #[rstest]
    fn test_fuzzy_search_num_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..10)
            .map(|i| SearchItem::new(&format!("Test Document {}", i), ""))
            .collect();

        assert_eq!(
            engine
                .search(&items, "test", 5, SearchLogic::default())
                .len(),
            5
        );
        assert_eq!(
            engine
                .search(&items, "test", 3, SearchLogic::default())
                .len(),
            3
        );
        assert_eq!(
            engine
                .search(&items, "test", 0, SearchLogic::default())
                .len(),
            0
        );
    }

    #[rstest]
    fn test_fuzzy_search_empty_query_limit(engine: FuzzySearchEngine) {
        let items: Vec<SearchItem> = (0..101)
            .map(|i| SearchItem::new(&format!("Document {}", i), ""))
            .collect();

        let results = engine.search(&items, "", 100, SearchLogic::default());
        assert_eq!(results.len(), 100);
    }

    #[rstest]
    fn test_fuzzy_search_num_larger_than_available(
        engine: FuzzySearchEngine,
        search_items: Vec<SearchItem>,
    ) {
        let results = engine.search(&search_items, "test", 100, SearchLogic::default());
        assert!(results.len() <= search_items.len());
        assert!(!results.is_empty());
    }

    // ===========================================
    // OR vs AND logic comparison tests
    // ===========================================

    #[rstest]
    fn test_or_search_matches_any_keyword(
        engine: FuzzySearchEngine,
        logic_test_items: Vec<SearchItem>,
    ) {
        // OR search: "rust python" should match all 3 items
        let results = engine.search(&logic_test_items, "rust python", 100, SearchLogic::Or);

        assert_eq!(results.len(), 3);
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Documentation"));
        assert!(titles.contains(&"Python Documentation"));
        assert!(titles.contains(&"Rust and Python"));
    }

    #[rstest]
    fn test_and_search_requires_all_keywords(
        engine: FuzzySearchEngine,
        logic_test_items: Vec<SearchItem>,
    ) {
        // AND search: "rust python" should only match "Rust and Python"
        let results = engine.search(&logic_test_items, "rust python", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust and Python");
    }

    #[rstest]
    fn test_single_keyword_same_for_or_and_and(
        engine: FuzzySearchEngine,
        logic_test_items: Vec<SearchItem>,
    ) {
        let or_results = engine.search(&logic_test_items, "rust", 100, SearchLogic::Or);
        let and_results = engine.search(&logic_test_items, "rust", 100, SearchLogic::And);

        assert_eq!(or_results.len(), and_results.len());
    }

    #[rstest]
    fn test_and_search_keyword_order_independent(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust Programming", ""),
            SearchItem::new("Programming Rust", ""),
            SearchItem::new("Python Language", ""),
        ];

        let results = engine.search(&items, "Programming Rust", 100, SearchLogic::And);

        assert_eq!(results.len(), 2);
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Programming"));
        assert!(titles.contains(&"Programming Rust"));
    }

    #[rstest]
    fn test_and_search_non_consecutive_keywords(engine: FuzzySearchEngine) {
        let items = vec![
            SearchItem::new("Rust hoge Programming", ""),
            SearchItem::new("Rust Programming", ""),
            SearchItem::new("Python Language", ""),
        ];

        let results = engine.search(&items, "Rust Programming", 100, SearchLogic::And);

        assert_eq!(results.len(), 2);
        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Programming"));
        assert!(titles.contains(&"Rust hoge Programming"));
    }

    // ===========================================
    // Default logic is OR
    // ===========================================

    #[rstest]
    fn test_default_logic_is_or(engine: FuzzySearchEngine, logic_test_items: Vec<SearchItem>) {
        let default_results = engine.search(
            &logic_test_items,
            "rust python",
            100,
            SearchLogic::default(),
        );
        let or_results = engine.search(&logic_test_items, "rust python", 100, SearchLogic::Or);

        assert_eq!(default_results.len(), or_results.len());
        assert_eq!(default_results.len(), 3);
    }
}
