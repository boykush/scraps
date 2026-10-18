use crate::nfc::nfc;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub key: String,
    pub search_text: String,
}

impl SearchItem {
    /// Only what names or summarises a scrap: fuzzy matching across a whole
    /// body matches nearly any keyword, and the body is what get is for.
    pub fn new(key: &str, title: Option<&str>, description: Option<&str>, tags: &[String]) -> Self {
        let search_text = [key]
            .into_iter()
            .chain(title)
            .chain(description)
            .chain(tags.iter().map(String::as_str))
            .collect::<Vec<&str>>()
            .join(" ");
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
        let item = SearchItem::new("Test Key", None, None, &[]);
        assert_eq!(item.key, "Test Key");
        assert_eq!(item.search_text, "Test Key");
    }

    #[test]
    fn test_search_item_with_title() {
        let item = SearchItem::new("tables/orders", Some("Customer Orders"), None, &[]);
        assert_eq!(item.key, "tables/orders");
        assert_eq!(item.search_text, "tables/orders Customer Orders");
    }

    #[test]
    fn test_search_item_with_description_and_tags() {
        let tags = ["Data".to_string(), "Sales/Orders".to_string()];
        let item = SearchItem::new("orders", None, Some("One row per order."), &tags);
        assert_eq!(
            item.search_text,
            "orders One row per order. Data Sales/Orders"
        );
    }
}
