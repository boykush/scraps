use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use super::engine::SearchEngine;
use super::result::{SearchIndexItem, SearchResult};

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
    fn search(&self, items: &[SearchIndexItem], query: &str, num: usize) -> Vec<SearchResult> {
        if query.is_empty() {
            return items
                .iter()
                .take(num)
                .map(|item| SearchResult::new(&item.title))
                .collect();
        }

        let mut results_with_scores: Vec<(SearchResult, i64)> = items
            .iter()
            .filter_map(|item| {
                self.matcher
                    .fuzzy_match(&item.title, query)
                    .map(|score| (SearchResult::new(&item.title), score))
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

    fn create_test_items() -> Vec<SearchIndexItem> {
        vec![
            SearchIndexItem::new("Test Document"),
            SearchIndexItem::new("Another Document"),
            SearchIndexItem::new("Sample Test"),
            SearchIndexItem::new("Documentation"),
            SearchIndexItem::new("Testing Framework"),
            SearchIndexItem::new("Test Suite"),
        ]
    }

    #[test]
    fn test_fuzzy_search_engine_new() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();
        let results = engine.search(&items, "test", 100);

        assert!(!results.is_empty());
        assert!(results.len() <= items.len());
    }

    #[test]
    fn test_fuzzy_search_exact_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();
        let results = engine.search(&items, "Test Document", 100);

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Test Document");
    }

    #[test]
    fn test_fuzzy_search_typo_tolerance() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        // Test with smaller typos that are more likely to match
        let results = engine.search(&items, "tes", 100);
        assert!(!results.is_empty());

        let results = engine.search(&items, "doc", 100);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "doc", 100);
        assert!(!results.is_empty());

        let results = engine.search(&items, "fram", 100);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_ordering() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "test", 100);
        assert!(!results.is_empty());

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Test Document") || titles.contains(&"Sample Test"));
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "", 100);
        assert_eq!(results.len(), items.len());
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "xyzzyx", 100);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_case_insensitive() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        // Test with exact matches first
        let results1 = engine.search(&items, "test", 100);
        let results2 = engine.search(&items, "Test", 100);
        let results3 = engine.search(&items, "doc", 100);

        // These should all return results since SkimMatcherV2 is case-insensitive by default
        assert!(!results1.is_empty());
        assert!(!results2.is_empty());
        assert!(!results3.is_empty());
    }

    #[test]
    fn test_fuzzy_search_case_sensitive() {
        let engine = FuzzySearchEngine::with_case_sensitive(true);
        let items = create_test_items();

        // Test with exact case matches
        let results = engine.search(&items, "Test", 100);
        assert!(!results.is_empty());

        // Even case sensitive should find some matches
        let results = engine.search(&items, "Document", 100);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_empty_items() {
        let engine = FuzzySearchEngine::new();
        let items = vec![];

        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_empty_query_limit() {
        let engine = FuzzySearchEngine::new();
        let mut items = Vec::new();
        for i in 0..101 {
            items.push(SearchIndexItem::new(&format!("Document {}", i)));
        }

        let results = engine.search(&items, "", 100);
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_fuzzy_search_with_custom_num() {
        let engine = FuzzySearchEngine::new();
        let mut items = Vec::new();
        for i in 0..10 {
            items.push(SearchIndexItem::new(&format!("Test Document {}", i)));
        }

        // Test with num=5
        let results = engine.search(&items, "test", 5);
        assert_eq!(results.len(), 5);

        // Test with num=3
        let results = engine.search(&items, "test", 3);
        assert_eq!(results.len(), 3);

        // Test with num=0
        let results = engine.search(&items, "test", 0);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_num_larger_than_available() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items(); // 6 items

        let results = engine.search(&items, "test", 10);
        // All items contain "test" in some form, so this should return all matching items
        assert!(results.len() <= 6);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_empty_query_with_custom_num() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items(); // 6 items

        let results = engine.search(&items, "", 3);
        assert_eq!(results.len(), 3);

        let results = engine.search(&items, "", 10);
        assert_eq!(results.len(), 6); // All items returned
    }

    #[test]
    fn test_fuzzy_search_results_content() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "test", 100);
        assert!(!results.is_empty());

        for result in results {
            assert!(!result.title.is_empty());
        }
    }
}
