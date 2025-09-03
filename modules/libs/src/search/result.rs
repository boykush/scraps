#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub title: String,
}

impl SearchResult {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchIndexItem {
    pub title: String,
}

impl SearchIndexItem {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_new() {
        let result = SearchResult::new("Test Title");
        assert_eq!(result.title, "Test Title");
    }

    #[test]
    fn test_search_index_item_new() {
        let item = SearchIndexItem::new("Test Title");
        assert_eq!(item.title, "Test Title");
    }
}
