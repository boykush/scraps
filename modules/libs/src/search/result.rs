#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub title: String,
    pub search_text: String,
}

impl SearchItem {
    pub fn new(title: &str, body: &str) -> Self {
        Self {
            title: title.to_string(),
            search_text: format!("{} {}", title, body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_item_new() {
        let item = SearchItem::new("Test Title", "body content");
        assert_eq!(item.title, "Test Title");
        assert_eq!(item.search_text, "Test Title body content");
    }
}
