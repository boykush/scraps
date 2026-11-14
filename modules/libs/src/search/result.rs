use crate::nfc::nfc;

/// How much a match counts by where it lands: what names a scrap outranks
/// what classifies it, which outranks what describes it.
const NAME_WEIGHT: i64 = 4;
const TAG_WEIGHT: i64 = 3;
const DESCRIPTION_WEIGHT: i64 = 1;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct SearchField {
    pub(super) text: String,
    pub(super) weight: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub key: String,
    pub(super) fields: Vec<SearchField>,
}

impl SearchItem {
    /// Only what names or summarises a scrap: fuzzy matching across a whole
    /// body matches nearly any keyword, and the body is what get is for.
    pub fn new(key: &str, title: Option<&str>, description: Option<&str>, tags: &[String]) -> Self {
        let names = [key].into_iter().chain(title).map(|t| (t, NAME_WEIGHT));
        let tags = tags.iter().map(|t| (t.as_str(), TAG_WEIGHT));
        let description = description.into_iter().map(|t| (t, DESCRIPTION_WEIGHT));
        let fields = names
            .chain(tags)
            .chain(description)
            .map(|(text, weight)| SearchField {
                text: fold(text),
                weight,
            })
            .collect();
        Self {
            key: key.to_string(),
            fields,
        }
    }
}

/// The form both sides are compared in, so neither normalization form nor
/// letter case decides a match.
pub(super) fn fold(text: &str) -> String {
    nfc(text).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(item: &SearchItem) -> Vec<(&str, i64)> {
        item.fields
            .iter()
            .map(|f| (f.text.as_str(), f.weight))
            .collect()
    }

    #[test]
    fn test_search_item_new() {
        let item = SearchItem::new("Test Key", None, None, &[]);
        assert_eq!(item.key, "Test Key");
        assert_eq!(texts(&item), [("test key", NAME_WEIGHT)]);
    }

    #[test]
    fn test_search_item_with_title() {
        let item = SearchItem::new("tables/orders", Some("Customer Orders"), None, &[]);
        assert_eq!(item.key, "tables/orders");
        assert_eq!(
            texts(&item),
            [
                ("tables/orders", NAME_WEIGHT),
                ("customer orders", NAME_WEIGHT)
            ]
        );
    }

    #[test]
    fn test_search_item_with_description_and_tags() {
        let tags = ["Data".to_string(), "Sales/Orders".to_string()];
        let item = SearchItem::new("orders", None, Some("One row per order."), &tags);
        assert_eq!(
            texts(&item),
            [
                ("orders", NAME_WEIGHT),
                ("data", TAG_WEIGHT),
                ("sales/orders", TAG_WEIGHT),
                ("one row per order.", DESCRIPTION_WEIGHT)
            ]
        );
    }
}
