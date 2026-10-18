use chrono::{DateTime, Utc};
use serde_json::Value;

const FRONTMATTER_VERIFIED_KEY: &str = "verified";
const HUMAN_ACTOR_PREFIX: &str = "human:";

/// One confirmation of a scrap's content, as the Open Knowledge Format's
/// `verified` records it: who confirmed it, and when.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Verification {
    pub by: String,
    pub at: DateTime<Utc>,
}

/// How far a scrap's content has been confirmed, derived from its
/// verifications the way OKF orders its trust tiers, lowest first.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum TrustTier {
    Unverified,
    MachineConfirmed,
    HumanReviewed,
}

impl TrustTier {
    pub fn of(verifications: &[Verification]) -> TrustTier {
        if verifications.is_empty() {
            TrustTier::Unverified
        } else if verifications.iter().any(Verification::is_human) {
            TrustTier::HumanReviewed
        } else {
            TrustTier::MachineConfirmed
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TrustTier::Unverified => "unverified",
            TrustTier::MachineConfirmed => "machine-confirmed",
            TrustTier::HumanReviewed => "human-reviewed",
        }
    }
}

impl Verification {
    pub fn is_human(&self) -> bool {
        self.by.starts_with(HUMAN_ACTOR_PREFIX)
    }
}

/// The readable entries of the frontmatter `verified`, newest first. OKF lets
/// a single verifier be a bare mapping, read here as a one-entry list. An
/// entry without a non-empty `by`, or whose `at` is not a datetime with an
/// explicit offset, is skipped rather than reported, like any frontmatter.
pub fn frontmatter_verifications(frontmatter: Option<&Value>) -> Vec<Verification> {
    let entries = match frontmatter.and_then(|fm| fm.get(FRONTMATTER_VERIFIED_KEY)) {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(entry @ Value::Object(_)) => vec![entry],
        _ => Vec::new(),
    };
    let mut verifications: Vec<Verification> =
        entries.into_iter().filter_map(verification).collect();
    verifications.sort_by_key(|v| std::cmp::Reverse(v.at));
    verifications
}

fn verification(entry: &Value) -> Option<Verification> {
    let by = entry
        .get("by")?
        .as_str()
        .map(str::trim)
        .filter(|by| !by.is_empty())?;
    let at = DateTime::parse_from_rfc3339(entry.get("at")?.as_str()?.trim()).ok()?;
    Some(Verification {
        by: by.to_string(),
        at: at.with_timezone(&Utc),
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
    fn it_reads_a_list_of_entries() {
        let fm = json!({"verified": [
            {"by": "human:alice", "at": "2026-10-17T00:00:00Z"}
        ]});
        assert_eq!(
            frontmatter_verifications(Some(&fm)),
            vec![Verification {
                by: "human:alice".to_string(),
                at: at("2026-10-17T00:00:00Z"),
            }]
        );
    }

    #[test]
    fn it_reads_a_bare_mapping_as_a_one_entry_list() {
        let fm = json!({"verified": {"by": "human:alice", "at": "2026-10-17T00:00:00Z"}});
        assert_eq!(frontmatter_verifications(Some(&fm)).len(), 1);
    }

    #[test]
    fn it_orders_entries_newest_first_across_offsets() {
        let fm = json!({"verified": [
            {"by": "human:a", "at": "2026-10-17T09:00:00+09:00"},
            {"by": "human:b", "at": "2026-10-17T01:00:00Z"},
            {"by": "human:c", "at": "2026-10-16T23:00:00Z"}
        ]});
        let by: Vec<String> = frontmatter_verifications(Some(&fm))
            .into_iter()
            .map(|v| v.by)
            .collect();
        assert_eq!(by, vec!["human:b", "human:a", "human:c"]);
    }

    #[test]
    fn it_normalizes_the_time_to_utc() {
        let fm = json!({"verified": {"by": "human:a", "at": "2026-10-17T09:00:00+09:00"}});
        assert_eq!(
            frontmatter_verifications(Some(&fm))[0].at,
            at("2026-10-17T00:00:00Z")
        );
    }

    #[rstest]
    #[case::no_by(json!({"at": "2026-10-17T00:00:00Z"}))]
    #[case::blank_by(json!({"by": "  ", "at": "2026-10-17T00:00:00Z"}))]
    #[case::non_string_by(json!({"by": 1, "at": "2026-10-17T00:00:00Z"}))]
    #[case::no_at(json!({"by": "human:a"}))]
    #[case::date_only(json!({"by": "human:a", "at": "2026-10-17"}))]
    #[case::no_offset(json!({"by": "human:a", "at": "2026-10-17T00:00:00"}))]
    #[case::non_string_at(json!({"by": "human:a", "at": 20261017}))]
    #[case::not_a_mapping(json!("human:a"))]
    fn it_skips_an_unreadable_entry(#[case] entry: Value) {
        let fm = json!({"verified": [entry, {"by": "human:ok", "at": "2026-10-17T00:00:00Z"}]});
        let by: Vec<String> = frontmatter_verifications(Some(&fm))
            .into_iter()
            .map(|v| v.by)
            .collect();
        assert_eq!(by, vec!["human:ok"]);
    }

    #[rstest]
    #[case::no_frontmatter(None)]
    #[case::no_key(Some(json!({"type": "Blog"})))]
    #[case::string(Some(json!({"verified": "yes"})))]
    #[case::null(Some(json!({"verified": null})))]
    fn it_has_no_verifications_without_readable_entries(#[case] fm: Option<Value>) {
        assert!(frontmatter_verifications(fm.as_ref()).is_empty());
    }

    #[rstest]
    #[case::none(json!({}), TrustTier::Unverified)]
    #[case::machine_only(
        json!({"verified": [{"by": "reference_agent/gemini-2.5-pro", "at": "2026-10-17T00:00:00Z"}]}),
        TrustTier::MachineConfirmed
    )]
    #[case::human_among_machines(
        json!({"verified": [
            {"by": "process:ci", "at": "2026-10-18T00:00:00Z"},
            {"by": "human:alice", "at": "2026-10-17T00:00:00Z"}
        ]}),
        TrustTier::HumanReviewed
    )]
    #[case::only_unreadable(
        json!({"verified": [{"by": "human:alice", "at": "2026-10-17"}]}),
        TrustTier::Unverified
    )]
    fn it_derives_the_trust_tier(#[case] fm: Value, #[case] expected: TrustTier) {
        assert_eq!(
            TrustTier::of(&frontmatter_verifications(Some(&fm))),
            expected
        );
    }
}
