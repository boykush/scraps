use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

const SECONDS_PER_DAY: i64 = 86_400;

/// Flag scraps whose newest `verified` confirmation is older than
/// `threshold_days`. A scrap nobody confirmed is not reported: the rule asks
/// for a confirmation to be renewed, never for a first one.
///
/// Opt-in (`[lint.stale_verified]` or `--rule stale-verified`): how long a
/// confirmation holds is a policy each wiki sets.
pub struct StaleVerifiedRule {
    pub threshold_days: u64,
    pub now_ts: i64,
}

impl LintRule for StaleVerifiedRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::StaleVerified
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let threshold_secs = (self.threshold_days as i64).saturating_mul(SECONDS_PER_DAY);
        let cutoff = self.now_ts.saturating_sub(threshold_secs);

        scraps
            .iter()
            .filter_map(|scrap| {
                // `verifications` is newest first.
                let newest = scrap.verifications().into_iter().next()?;
                let ts = newest.at.timestamp();
                if ts >= cutoff {
                    return None;
                }
                let age_days = (self.now_ts - ts) / SECONDS_PER_DAY;
                Some(LintWarning {
                    rule_name: LintRuleName::StaleVerified,
                    scrap_path: scrap_relative_path(scrap),
                    message: format!("last verified {} days ago by {}", age_days, newest.by),
                    source: None,
                    span: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-14T00:00:00Z
    const NOW: i64 = 1_794_614_400;

    fn check(text: &str, threshold_days: u64) -> Vec<LintWarning> {
        let scraps = vec![Scrap::new("note", &Some("ai".into()), text)];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);
        StaleVerifiedRule {
            threshold_days,
            now_ts: NOW,
        }
        .check(&scraps, &backlinks_map, &tags)
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-01/example/EX-01
    #[test]
    fn flag_confirmation_older_than_threshold() {
        let warnings = check(
            "---\nverified:\n  - by: human:alice\n    at: 2026-04-28T00:00:00Z\n---\nbody",
            180,
        );

        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::StaleVerified);
        assert_eq!(warnings[0].scrap_path, "ai/note.md");
        assert_eq!(
            warnings[0].message,
            "last verified 200 days ago by human:alice"
        );
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-01/example/EX-02
    #[test]
    fn skip_confirmation_within_threshold() {
        let warnings = check(
            "---\nverified:\n  - by: human:alice\n    at: 2026-10-15T00:00:00Z\n---\nbody",
            180,
        );

        assert!(warnings.is_empty());
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-02/example/EX-01
    #[test]
    fn newest_confirmation_decides() {
        let warnings = check(
            "---\nverified:\n  - by: human:alice\n    at: 2025-01-01T00:00:00Z\n  - by: human:bob\n    at: 2026-11-01T00:00:00Z\n---\nbody",
            180,
        );

        assert!(warnings.is_empty());
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-03/example/EX-01
    #[test]
    fn skip_scrap_nobody_confirmed() {
        assert!(check("body", 180).is_empty());
        assert!(check("---\ntitle: Note\n---\nbody", 180).is_empty());
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-03/example/EX-02
    #[test]
    fn skip_unreadable_confirmation() {
        let warnings = check("---\nverified:\n  - by: human:alice\n---\nbody", 180);

        assert!(warnings.is_empty());
    }
}
