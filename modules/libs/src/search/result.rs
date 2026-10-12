use crate::nfc::nfc;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub key: String,
    pub search_text: String,
}

impl SearchItem {
    /// `title` is the scrap's own title when it has one. A scrap named only
    /// by its key is searched as key and body, so its text stays what it was
    /// before titles existed.
    pub fn new(key: &str, title: Option<&str>, body: &str) -> Self {
        let search_text = match title {
            Some(title) => format!("{key} {title} {body}"),
            None => format!("{key} {body}"),
        };
        Self {
            key: key.to_string(),
            search_text: nfc(&search_text).into_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_item_new() {
        let item = SearchItem::new("Test Key", None, "body content");
        assert_eq!(item.key, "Test Key");
        assert_eq!(item.search_text, "Test Key body content");
    }

    #[test]
    fn test_search_item_with_title() {
        let item = SearchItem::new("tables/orders", Some("Customer Orders"), "body");
        assert_eq!(item.key, "tables/orders");
        assert_eq!(item.search_text, "tables/orders Customer Orders body");
    }
}
