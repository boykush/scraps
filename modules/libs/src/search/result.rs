#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
}

impl SearchResult {
    pub fn new(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchIndexItem {
    pub title: String,
    pub url: String,
}

impl SearchIndexItem {
    pub fn new(title: &str, url: &str) -> Self {
        Self {
            title: title.to_string(),
            url: url.to_string(),
        }
    }
}

impl From<SearchIndexItem> for SearchResult {
    fn from(item: SearchIndexItem) -> Self {
        SearchResult::new(&item.title, &item.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_new() {
        let result = SearchResult::new("Test Title", "http://example.com");
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "http://example.com");
    }

    #[test]
    fn test_search_index_item_new() {
        let item = SearchIndexItem::new("Test Title", "http://example.com");
        assert_eq!(item.title, "Test Title");
        assert_eq!(item.url, "http://example.com");
    }

    #[test]
    fn test_from_search_index_item() {
        let item = SearchIndexItem::new("Test Title", "http://example.com");
        let result: SearchResult = item.into();
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "http://example.com");
    }
}