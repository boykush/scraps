#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub title: String,
}

impl SearchItem {
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
    fn test_search_item_new() {
        let item = SearchItem::new("Test Title");
        assert_eq!(item.title, "Test Title");
    }
}
