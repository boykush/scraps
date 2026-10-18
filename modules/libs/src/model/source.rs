use chrono::{DateTime, Utc};
use serde_json::Value;

const FRONTMATTER_SOURCES_KEY: &str = "sources";

/// One material a scrap's content derives from, as the Open Knowledge
/// Format's `sources` records it. `resource` is kept as written: OKF lets it
/// be a URL, a path or a scope descriptor without marking which.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Source {
    pub resource: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
}

/// The readable entries of the frontmatter `sources`, in the order written.
/// A bare mapping is read as a one-entry list, as OKF allows for `verified`.
/// An entry without a non-empty `resource` is skipped; an optional key of the
/// wrong shape is dropped and the entry kept.
pub fn frontmatter_sources(frontmatter: Option<&Value>) -> Vec<Source> {
    let entries = match frontmatter.and_then(|fm| fm.get(FRONTMATTER_SOURCES_KEY)) {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(entry @ Value::Object(_)) => vec![entry],
        _ => Vec::new(),
    };
    entries.into_iter().filter_map(source).collect()
}

fn source(entry: &Value) -> Option<Source> {
    Some(Source {
        resource: non_empty_string(entry, "resource")?,
        id: non_empty_string(entry, "id"),
        title: non_empty_string(entry, "title"),
        author: non_empty_string(entry, "author"),
        last_modified: entry
            .get("last_modified")
            .and_then(Value::as_str)
            .and_then(|at| DateTime::parse_from_rfc3339(at.trim()).ok())
            .map(|at| at.with_timezone(&Utc)),
    })
}

fn non_empty_string(entry: &Value, key: &str) -> Option<String> {
    entry
        .get(key)?
        .as_str()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    fn resources(fm: &Value) -> Vec<String> {
        frontmatter_sources(Some(fm))
            .into_iter()
            .map(|s| s.resource)
            .collect()
    }

    #[test]
    fn it_reads_every_key_of_an_entry() {
        let fm = json!({"sources": [{
            "resource": "https://example.com/ga4",
            "id": "ga4-schema",
            "title": "GA4 schema",
            "author": "team:ga4-docs",
            "last_modified": "2026-05-30T09:00:00+09:00"
        }]});
        assert_eq!(
            frontmatter_sources(Some(&fm)),
            vec![Source {
                resource: "https://example.com/ga4".to_string(),
                id: Some("ga4-schema".to_string()),
                title: Some("GA4 schema".to_string()),
                author: Some("team:ga4-docs".to_string()),
                last_modified: Some(
                    DateTime::parse_from_rfc3339("2026-05-30T00:00:00Z")
                        .unwrap()
                        .with_timezone(&Utc)
                ),
            }]
        );
    }

    #[test]
    fn it_keeps_the_written_order() {
        let fm = json!({"sources": [
            {"resource": "b"}, {"resource": "a"}, {"resource": "c"}
        ]});
        assert_eq!(resources(&fm), vec!["b", "a", "c"]);
    }

    #[test]
    fn it_reads_a_bare_mapping_as_a_one_entry_list() {
        let fm = json!({"sources": {"resource": "https://example.com"}});
        assert_eq!(resources(&fm), vec!["https://example.com"]);
    }

    #[test]
    fn it_keeps_any_resource_as_written() {
        let fm = json!({"sources": [
            {"resource": " references/schema.md "},
            {"resource": "all queries in BigQuery project X"}
        ]});
        assert_eq!(
            resources(&fm),
            vec!["references/schema.md", "all queries in BigQuery project X"]
        );
    }

    #[rstest]
    #[case::no_resource(json!({"title": "t"}))]
    #[case::blank_resource(json!({"resource": "  "}))]
    #[case::non_string_resource(json!({"resource": 1}))]
    #[case::bare_string(json!("https://example.com"))]
    fn it_skips_an_unreadable_entry(#[case] entry: Value) {
        let fm = json!({"sources": [entry, {"resource": "ok"}]});
        assert_eq!(resources(&fm), vec!["ok"]);
    }

    #[rstest]
    #[case::date_only(json!({"resource": "r", "last_modified": "2026-05-30", "title": 1}))]
    #[case::no_offset(json!({"resource": "r", "last_modified": "2026-05-30T00:00:00", "id": ""}))]
    fn it_drops_an_optional_key_of_the_wrong_shape(#[case] entry: Value) {
        let fm = json!({"sources": [entry]});
        assert_eq!(
            frontmatter_sources(Some(&fm)),
            vec![Source {
                resource: "r".to_string(),
                id: None,
                title: None,
                author: None,
                last_modified: None,
            }]
        );
    }

    #[rstest]
    #[case::no_frontmatter(None)]
    #[case::no_key(Some(json!({"type": "Blog"})))]
    #[case::string(Some(json!({"sources": "https://example.com"})))]
    #[case::null(Some(json!({"sources": null})))]
    fn it_has_no_sources_without_readable_entries(#[case] fm: Option<Value>) {
        assert!(frontmatter_sources(fm.as_ref()).is_empty());
    }
}
