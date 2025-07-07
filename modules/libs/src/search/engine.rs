use super::result::{SearchIndexItem, SearchResult};

pub trait SearchEngine {
    fn search(&self, items: &[SearchIndexItem], query: &str) -> Vec<SearchResult>;
}

pub struct SimpleStringSearchEngine;

impl SimpleStringSearchEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleStringSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine for SimpleStringSearchEngine {
    fn search(&self, items: &[SearchIndexItem], query: &str) -> Vec<SearchResult> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();

        items
            .iter()
            .filter(|item| item.title.to_lowercase().contains(&query_lower))
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
        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_simple_string_search_engine_default() {
        let engine = SimpleStringSearchEngine::default();
        let items = create_test_items();
        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        
        let results = engine.search(&items, "TEST");
        assert_eq!(results.len(), 2);
        
        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 2);
        
        let results = engine.search(&items, "Test");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_partial_match() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        
        let results = engine.search(&items, "doc");
        assert_eq!(results.len(), 2); // "Test Document" and "Documentation"
    }

    #[test]
    fn test_search_no_match() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        
        let results = engine.search(&items, "nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_query() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        
        let results = engine.search(&items, "");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_empty_items() {
        let engine = SimpleStringSearchEngine::new();
        let items = vec![];
        
        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_results_content() {
        let engine = SimpleStringSearchEngine::new();
        let items = create_test_items();
        
        let results = engine.search(&items, "test");
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
        
        let results = engine.search(&items, "test");
        assert_eq!(results.len(), 3);
        
        let results = engine.search(&items, "test-");
        assert_eq!(results.len(), 1);
        
        let results = engine.search(&items, "test_");
        assert_eq!(results.len(), 1);
    }
}