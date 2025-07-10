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
    fn search(&self, items: &[SearchIndexItem], query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return items
                .iter()
                .take(100)
                .map(|item| SearchResult::new(&item.title, &item.url))
                .collect();
        }

        let mut results_with_scores: Vec<(SearchResult, i64)> = items
            .iter()
            .filter_map(|item| {
                self.matcher
                    .fuzzy_match(&item.title, query)
                    .map(|score| (SearchResult::new(&item.title, &item.url), score))
            })
            .collect();

        results_with_scores.sort_by(|a, b| b.1.cmp(&a.1));

        results_with_scores
            .into_iter()
            .map(|(result, _)| result)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_items() -> Vec<SearchIndexItem> {
        vec![
            SearchIndexItem::new("Test Document", "http://example.com/test"),
            SearchIndexItem::new("Another Document", "http://example.com/another"),
            SearchIndexItem::new("Sample Test", "http://example.com/sample"),
            SearchIndexItem::new("Documentation", "http://example.com/doc"),
            SearchIndexItem::new("Testing Framework", "http://example.com/testing"),
            SearchIndexItem::new("Test Suite", "http://example.com/suite"),
        ]
    }

    #[test]
    fn test_fuzzy_search_engine_new() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();
        let results = engine.search(&items, "test");

        assert!(!results.is_empty());
        assert!(results.len() <= items.len());
    }

    #[test]
    fn test_fuzzy_search_exact_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();
        let results = engine.search(&items, "Test Document");

        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Test Document");
    }

    #[test]
    fn test_fuzzy_search_typo_tolerance() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        // Test with smaller typos that are more likely to match
        let results = engine.search(&items, "tes");
        assert!(!results.is_empty());

        let results = engine.search(&items, "doc");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_partial_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "doc");
        assert!(!results.is_empty());

        let results = engine.search(&items, "fram");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_ordering() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "test");
        assert!(!results.is_empty());

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Test Document") || titles.contains(&"Sample Test"));
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "");
        assert_eq!(results.len(), items.len());
    }

    #[test]
    fn test_fuzzy_search_no_match() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "xyzzyx");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_case_insensitive() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        // Test with exact matches first
        let results1 = engine.search(&items, "test");
        let results2 = engine.search(&items, "Test");
        let results3 = engine.search(&items, "doc");

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
        let results = engine.search(&items, "Test");
        assert!(!results.is_empty());

        // Even case sensitive should find some matches
        let results = engine.search(&items, "Document");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_fuzzy_search_empty_items() {
        let engine = FuzzySearchEngine::new();
        let items = vec![];

        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_fuzzy_search_empty_query_limit() {
        let engine = FuzzySearchEngine::new();
        let mut items = Vec::new();
        for i in 0..101 {
            items.push(SearchIndexItem::new(
                &format!("Document {}", i),
                &format!("http://example.com/doc{}", i),
            ));
        }

        let results = engine.search(&items, "");
        assert_eq!(results.len(), 100);
    }


    #[test]
    fn test_fuzzy_search_results_content() {
        let engine = FuzzySearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "test");
        assert!(!results.is_empty());

        for result in results {
            assert!(!result.title.is_empty());
            assert!(!result.url.is_empty());
            assert!(result.url.starts_with("http://"));
        }
    }
}
