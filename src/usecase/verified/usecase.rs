use chrono::{DateTime, Utc};
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::verification::Verification;

use crate::error::ScrapsResult;
use crate::usecase::scrap::scrap_ref::ScrapRef;

const SECONDS_PER_DAY: i64 = 86_400;

/// Whose confirmations `scraps verified` counts. A person's say what they
/// still understand, a machine's what was checked against its sources: two
/// clocks kept in one `verified`, told apart by the actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmerFilter {
    Human,
    Machine,
    Any,
}

impl ConfirmerFilter {
    fn matches(self, verification: &Verification) -> bool {
        match self {
            ConfirmerFilter::Any => true,
            ConfirmerFilter::Human => verification.is_human(),
            ConfirmerFilter::Machine => !verification.is_human(),
        }
    }
}

/// A scrap with the newest confirmation the filter counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedResult {
    pub scrap: ScrapRef,
    pub by: String,
    pub at: DateTime<Utc>,
    pub age_days: i64,
}

pub struct VerifiedUsecase;

impl VerifiedUsecase {
    pub fn new() -> VerifiedUsecase {
        VerifiedUsecase
    }

    /// One result per scrap that `confirmer` confirmed, oldest confirmation
    /// first. `older_than_days` keeps only those confirmed longer ago.
    pub fn execute(
        &self,
        scraps: &[Scrap],
        confirmer: ConfirmerFilter,
        older_than_days: Option<u64>,
        now_ts: i64,
    ) -> ScrapsResult<Vec<VerifiedResult>> {
        let cutoff = older_than_days
            .map(|days| now_ts.saturating_sub((days as i64).saturating_mul(SECONDS_PER_DAY)));

        let mut results: Vec<VerifiedResult> = scraps
            .iter()
            .filter_map(|scrap| {
                // `verifications` is newest first.
                let newest = scrap
                    .verifications()
                    .into_iter()
                    .find(|v| confirmer.matches(v))?;
                let ts = newest.at.timestamp();
                if cutoff.is_some_and(|cutoff| ts >= cutoff) {
                    return None;
                }
                Some(VerifiedResult {
                    scrap: scrap.into(),
                    by: newest.by,
                    at: newest.at,
                    age_days: (now_ts - ts) / SECONDS_PER_DAY,
                })
            })
            .collect();

        results.sort_by_key(|result| (result.at, result.scrap.sort_key()));

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-14T00:00:00Z
    const NOW: i64 = 1_794_614_400;

    fn scrap(stem: &str, verified: &[(&str, &str)]) -> Scrap {
        let entries: String = verified
            .iter()
            .map(|(by, at)| format!("  - by: {by}\n    at: {at}\n"))
            .collect();
        Scrap::new(stem, &None, &format!("---\nverified:\n{entries}---\nbody"))
    }

    fn keys(results: &[VerifiedResult]) -> Vec<String> {
        results.iter().map(|r| r.scrap.key.to_string()).collect()
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-01/example/EX-01
    #[test]
    fn lists_newest_confirmation_with_its_age() {
        let scraps = vec![scrap(
            "note",
            &[
                ("human:alice", "2025-01-01T00:00:00Z"),
                ("human:bob", "2026-04-28T00:00:00Z"),
            ],
        )];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Any, None, NOW)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].by, "human:bob");
        assert_eq!(results[0].at.to_rfc3339(), "2026-04-28T00:00:00+00:00");
        assert_eq!(results[0].age_days, 200);
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-01/example/EX-02
    #[test]
    fn orders_oldest_confirmation_first() {
        let scraps = vec![
            scrap("recent", &[("human:alice", "2026-11-01T00:00:00Z")]),
            scrap("oldest", &[("human:alice", "2025-01-01T00:00:00Z")]),
            scrap("middle", &[("human:alice", "2026-04-28T00:00:00Z")]),
        ];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Any, None, NOW)
            .unwrap();

        assert_eq!(keys(&results), vec!["oldest", "middle", "recent"]);
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-02/example/EX-01
    #[test]
    fn human_filter_ignores_a_newer_machine_confirmation() {
        let scraps = vec![scrap(
            "note",
            &[
                ("process:ci", "2026-11-01T00:00:00Z"),
                ("human:alice", "2026-04-28T00:00:00Z"),
            ],
        )];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Human, None, NOW)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].by, "human:alice");
        assert_eq!(results[0].age_days, 200);
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-02/example/EX-02
    #[test]
    fn machine_filter_ignores_a_newer_human_confirmation() {
        let scraps = vec![scrap(
            "note",
            &[
                ("human:alice", "2026-11-01T00:00:00Z"),
                ("claude-code/opus", "2026-04-28T00:00:00Z"),
            ],
        )];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Machine, None, NOW)
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].by, "claude-code/opus");
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-03/example/EX-01
    #[test]
    fn leaves_out_scraps_the_confirmer_never_confirmed() {
        let scraps = vec![
            Scrap::new("plain", &None, "body"),
            scrap("by_machine", &[("process:ci", "2020-01-01T00:00:00Z")]),
            scrap("by_human", &[("human:alice", "2020-01-01T00:00:00Z")]),
        ];
        let usecase = VerifiedUsecase::new();

        let human = usecase
            .execute(&scraps, ConfirmerFilter::Human, None, NOW)
            .unwrap();
        let machine = usecase
            .execute(&scraps, ConfirmerFilter::Machine, None, NOW)
            .unwrap();

        assert_eq!(keys(&human), vec!["by_human"]);
        assert_eq!(keys(&machine), vec!["by_machine"]);
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-04/example/EX-01
    #[test]
    fn older_than_keeps_only_confirmations_past_the_days() {
        let scraps = vec![
            scrap("due", &[("human:alice", "2026-04-28T00:00:00Z")]),
            scrap("fresh", &[("human:alice", "2026-10-15T00:00:00Z")]),
        ];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Human, Some(180), NOW)
            .unwrap();

        assert_eq!(keys(&results), vec!["due"]);
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-04/example/EX-02
    #[test]
    fn older_than_zero_keeps_every_confirmation_in_the_past() {
        let scraps = vec![scrap("note", &[("human:alice", "2026-11-13T00:00:00Z")])];

        let results = VerifiedUsecase::new()
            .execute(&scraps, ConfirmerFilter::Human, Some(0), NOW)
            .unwrap();

        assert_eq!(keys(&results), vec!["note"]);
    }
}
