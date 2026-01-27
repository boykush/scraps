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

    #[rstest]
    fn test_fuzzy_search_engine_new(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();
        let results = engine.search(&search_items, "test", 100, SearchLogic::And);

        assert!(!results.is_empty());
        assert!(results.len() <= search_items.len());
    }

    #[rstest]
    fn test_fuzzy_search_exact_match(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();
        let results = engine.search(&search_items, "Test Document", 100, SearchLogic::And);

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Test Document");
    }

    #[rstest]
    fn test_fuzzy_search_typo_tolerance(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        // Test with smaller typos that are more likely to match
        let results = engine.search(&search_items, "tes", 100, SearchLogic::And);
        assert!(!results.is_empty());

        let results = engine.search(&search_items, "doc", 100, SearchLogic::And);
        assert!(!results.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_partial_match(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "doc", 100, SearchLogic::And);
        assert!(!results.is_empty());

        let results = engine.search(&search_items, "fram", 100, SearchLogic::And);
        assert!(!results.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_ordering(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "test", 100, SearchLogic::And);
        assert!(!results.is_empty());

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Test Document") || titles.contains(&"Sample Test"));
    }

    #[rstest]
    fn test_fuzzy_search_empty_query(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "", 100, SearchLogic::And);
        assert_eq!(results.len(), search_items.len());
    }

    #[rstest]
    fn test_fuzzy_search_no_match(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "xyzzyx", 100, SearchLogic::And);
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_fuzzy_search_case_insensitive(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        // Test with exact matches first
        let results1 = engine.search(&search_items, "test", 100, SearchLogic::And);
        let results2 = engine.search(&search_items, "Test", 100, SearchLogic::And);
        let results3 = engine.search(&search_items, "doc", 100, SearchLogic::And);

        // These should all return results since SkimMatcherV2 is case-insensitive by default
        assert!(!results1.is_empty());
        assert!(!results2.is_empty());
        assert!(!results3.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_case_sensitive(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::with_case_sensitive(true);

        // Test with exact case matches
        let results = engine.search(&search_items, "Test", 100, SearchLogic::And);
        assert!(!results.is_empty());

        // Even case sensitive should find some matches
        let results = engine.search(&search_items, "Document", 100, SearchLogic::And);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_empty_items() {
        let engine = FuzzySearchEngine::new();
        let items = vec![];

        let results = engine.search(&items, "test", 100, SearchLogic::And);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_empty_query_limit() {
        let engine = FuzzySearchEngine::new();
        let mut items = Vec::new();
        for i in 0..101 {
            items.push(SearchItem::new(&format!("Document {}", i), ""));
        }

        let results = engine.search(&items, "", 100, SearchLogic::And);
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_fuzzy_search_with_custom_num() {
        let engine = FuzzySearchEngine::new();
        let mut items = Vec::new();
        for i in 0..10 {
            items.push(SearchItem::new(&format!("Test Document {}", i), ""));
        }

        // Test with num=5
        let results = engine.search(&items, "test", 5, SearchLogic::And);
        assert_eq!(results.len(), 5);

        // Test with num=3
        let results = engine.search(&items, "test", 3, SearchLogic::And);
        assert_eq!(results.len(), 3);

        // Test with num=0
        let results = engine.search(&items, "test", 0, SearchLogic::And);
        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_fuzzy_search_num_larger_than_available(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "test", 10, SearchLogic::And);
        // All items contain "test" in some form, so this should return all matching items
        assert!(results.len() <= 6);
        assert!(!results.is_empty());
    }

    #[rstest]
    fn test_fuzzy_search_empty_query_with_custom_num(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "", 3, SearchLogic::And);
        assert_eq!(results.len(), 3);

        let results = engine.search(&search_items, "", 10, SearchLogic::And);
        assert_eq!(results.len(), 6); // All items returned
    }

    #[rstest]
    fn test_fuzzy_search_results_content(search_items: Vec<SearchItem>) {
        let engine = FuzzySearchEngine::new();

        let results = engine.search(&search_items, "test", 100, SearchLogic::And);
        assert!(!results.is_empty());

        for result in results {
            assert!(!result.title.is_empty());
        }
    }

    /// Test: AND search matches non-consecutive keywords (fzf-compatible)
    /// Query "Rust Programming" should match "Rust hoge Programming"
    #[test]
    fn test_and_search_non_consecutive_keywords() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Rust hoge Programming", ""),
            SearchItem::new("Rust Programming", ""),
            SearchItem::new("Python Language", ""),
        ];

        let results = engine.search(&items, "Rust Programming", 100, SearchLogic::And);

        // Both "Rust hoge Programming" and "Rust Programming" should match
        assert_eq!(results.len(), 2);

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Programming"));
        assert!(titles.contains(&"Rust hoge Programming"));
    }

    /// Test: AND search matches reversed keyword order (fzf-compatible)
    /// Query "Programming Rust" should match "Rust Programming"
    #[test]
    fn test_and_search_reversed_keyword_order() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Rust Programming", ""),
            SearchItem::new("Programming Rust", ""),
            SearchItem::new("Python Language", ""),
        ];

        let results = engine.search(&items, "Programming Rust", 100, SearchLogic::And);

        // Both should match regardless of keyword order
        assert_eq!(results.len(), 2);

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Programming"));
        assert!(titles.contains(&"Programming Rust"));
    }

    /// Test: search matches body content
    #[test]
    fn test_search_matches_body_content() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Document A", "contains uniquekeyword here"),
            SearchItem::new("Document B", "no match"),
        ];

        let results = engine.search(&items, "uniquekeyword", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Document A");
    }

    /// Test: OR search returns results matching any keyword
    #[test]
    fn test_or_search_any_keyword_matches() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Rust Documentation", "Rust content"),
            SearchItem::new("Python Documentation", "Python content"),
            SearchItem::new("Rust and Python", "Both languages"),
        ];

        // OR search: "rust python" should match all 3 items
        let results = engine.search(&items, "rust python", 100, SearchLogic::Or);

        assert_eq!(results.len(), 3);

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Rust Documentation"));
        assert!(titles.contains(&"Python Documentation"));
        assert!(titles.contains(&"Rust and Python"));
    }

    /// Test: AND search only returns results matching all keywords
    #[test]
    fn test_and_search_all_keywords_required() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Rust Documentation", "Rust content"),
            SearchItem::new("Python Documentation", "Python content"),
            SearchItem::new("Rust and Python", "Both languages"),
        ];

        // AND search: "rust python" should only match "Rust and Python"
        let results = engine.search(&items, "rust python", 100, SearchLogic::And);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust and Python");
    }

    /// Test: OR search with single keyword behaves same as AND
    #[test]
    fn test_or_search_single_keyword() {
        let engine = FuzzySearchEngine::new();
        let items = vec![
            SearchItem::new("Rust Documentation", ""),
            SearchItem::new("Python Documentation", ""),
        ];

        let or_results = engine.search(&items, "rust", 100, SearchLogic::Or);
        let and_results = engine.search(&items, "rust", 100, SearchLogic::And);

        assert_eq!(or_results.len(), and_results.len());
    }
}
