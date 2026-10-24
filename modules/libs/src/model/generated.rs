use chrono::{DateTime, Utc};
use serde_json::Value;

const FRONTMATTER_GENERATED_KEY: &str = "generated";

/// How a scrap's current content was produced, as the Open Knowledge
/// Format's `generated` records it: who wrote it, and when it last changed
/// in a meaningful way.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Generated {
    pub by: String,
    pub at: Option<DateTime<Utc>>,
}

/// The frontmatter `generated`, when it is a mapping with a non-empty `by`.
/// OKF requires only `by`, so an `at` that is not a datetime with an
/// explicit offset is dropped and the rest kept.
pub fn frontmatter_generated(frontmatter: Option<&Value>) -> Option<Generated> {
    let entry = frontmatter?.get(FRONTMATTER_GENERATED_KEY)?;
    let by = entry
        .get("by")?
        .as_str()
        .map(str::trim)
        .filter(|by| !by.is_empty())?;
    let at = entry
        .get("at")
        .and_then(Value::as_str)
        .and_then(|at| DateTime::parse_from_rfc3339(at.trim()).ok())
        .map(|at| at.with_timezone(&Utc));
    Some(Generated {
        by: by.to_string(),
        at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    fn at(rfc3339: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(rfc3339)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn it_reads_by_and_at_with_the_time_in_utc() {
        let fm = json!({"generated": {
            "by": " reference_agent/gemini-2.5-pro ",
            "at": "2026-06-21T07:53:05+09:00"
        }});
        assert_eq!(
            frontmatter_generated(Some(&fm)),
            Some(Generated {
                by: "reference_agent/gemini-2.5-pro".to_string(),
                at: Some(at("2026-06-20T22:53:05Z")),
            })
        );
    }

    #[rstest]
    #[case::no_at(json!({"by": "human:alice"}))]
    #[case::date_only(json!({"by": "human:alice", "at": "2026-06-20"}))]
    #[case::no_offset(json!({"by": "human:alice", "at": "2026-06-20T22:53:05"}))]
    #[case::non_string_at(json!({"by": "human:alice", "at": 20260620}))]
    fn it_keeps_by_without_a_readable_at(#[case] generated: Value) {
        let fm = json!({ "generated": generated });
        assert_eq!(
            frontmatter_generated(Some(&fm)),
            Some(Generated {
                by: "human:alice".to_string(),
                at: None,
            })
        );
    }

    #[rstest]
    #[case::no_frontmatter(None)]
    #[case::no_key(Some(json!({"type": "Blog"})))]
    #[case::no_by(Some(json!({"generated": {"at": "2026-06-20T22:53:05Z"}})))]
    #[case::blank_by(Some(json!({"generated": {"by": "  "}})))]
    #[case::non_string_by(Some(json!({"generated": {"by": 1}})))]
    #[case::string(Some(json!({"generated": "human:alice"})))]
    #[case::list(Some(json!({"generated": [{"by": "human:alice"}]})))]
    #[case::null(Some(json!({"generated": null})))]
    fn it_has_none_without_a_readable_by(#[case] fm: Option<Value>) {
        assert_eq!(frontmatter_generated(fm.as_ref()), None);
    }
}
