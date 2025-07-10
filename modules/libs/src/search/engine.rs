use super::result::{SearchIndexItem, SearchResult};

pub trait SearchEngine {
    fn search(&self, items: &[SearchIndexItem], query: &str, num: usize) -> Vec<SearchResult>;
}

pub struct SimpleStringSearchEngine;

impl SimpleStringSearchEngine {
    pub fn new() -> Self {
        Self
    }
}

impl SearchEngine for SimpleStringSearchEngine {
    fn search(&self, items: &[SearchIndexItem], query: &str, num: usize) -> Vec<SearchResult> {
        if query.is_empty() {
            return items
                .iter()
                .take(num)
                .map(|item| SearchResult::new(&item.title, &item.url))
                .collect();
        }

        let query_lower = query.to_lowercase();

        items
            .iter()
            .filter(|item| item.title.to_lowercase().contains(&query_lower))
            .take(num)
            .map(|item| SearchResult::new(&item.title, &item.url))
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
        ]
    }

    #[test]
    fn test_simple_string_search_engine_new() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "TEST", 100);
        assert_eq!(results.len(), 2);

        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 2);

        let results = engine.search(&items, "Test", 100);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_partial_match() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "doc", 100);
        assert_eq!(results.len(), 3); // "Test Document", "Another Document", and "Documentation"
    }

    #[test]
    fn test_search_no_match() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "nonexistent", 100);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_query() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "", 100);
        assert_eq!(results.len(), items.len());
    }

    #[test]
    fn test_search_empty_items() {
        let engine = SimpleStringSearchEngine::new();
        let items = vec![];

        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_results_content() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();

        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 2);

        let titles: Vec<&str> = results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"Test Document"));
        assert!(titles.contains(&"Sample Test"));

        let urls: Vec<&str> = results.iter().map(|r| r.url.as_str()).collect();
        assert!(urls.contains(&"http://example.com/test"));
        assert!(urls.contains(&"http://example.com/sample"));
    }

    #[test]
    fn test_search_special_characters() {
        let engine = SimpleStringSearchEngine::new();
        let items = vec![
            SearchIndexItem::new("Test-Document", "http://example.com/test"),
            SearchIndexItem::new("Test_Document", "http://example.com/test2"),
            SearchIndexItem::new("Test Document", "http://example.com/test3"),
        ];

        let results = engine.search(&items, "test", 100);
        assert_eq!(results.len(), 3);

        let results = engine.search(&items, "test-", 100);
        assert_eq!(results.len(), 1);

        let results = engine.search(&items, "test_", 100);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_empty_query_limit() {
        let engine = SimpleStringSearchEngine::new();
        let mut items = Vec::new();
        for i in 0..101 {
            items.push(SearchIndexItem::new(
                &format!("Document {}", i),
                &format!("http://example.com/doc{}", i),
            ));
        }

        let results = engine.search(&items, "", 100);
        assert_eq!(results.len(), 100);
    }

    #[test]
    fn test_search_with_custom_num() {
        let engine = SimpleStringSearchEngine::new();
        let mut items = Vec::new();
        for i in 0..10 {
            items.push(SearchIndexItem::new(
                &format!("Test Document {}", i),
                &format!("http://example.com/test{}", i),
            ));
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
    fn test_search_num_larger_than_available() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items(); // 4 items

        let results = engine.search(&items, "test", 10);
        assert_eq!(results.len(), 2); // Only 2 items match "test"
    }

    #[test]
    fn test_search_empty_query_with_custom_num() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items(); // 4 items

        let results = engine.search(&items, "", 2);
        assert_eq!(results.len(), 2);

        let results = engine.search(&items, "", 10);
        assert_eq!(results.len(), 4); // All items returned
    }
}
