use rayon::prelude::*;
use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::{error::ScrapsResult, usecase::build::model::backlinks_map::BacklinksMap};

use super::{
    rule::{LintRule, LintRuleName, LintWarning},
    rules::{
        broken_heading_ref::BrokenHeadingRefRule, broken_link::BrokenLinkRule,
        dead_end::DeadEndRule, lonely::LonelyRule, overlinking::OverlinkingRule,
        self_link::SelfLinkRule,
    },
};

pub struct LintUsecase;

impl LintUsecase {
    pub fn new() -> LintUsecase {
        LintUsecase
    }

    /// Run lint rules over `scraps` and return collected warnings.
    ///
    /// `rule_names` selects which rules to run:
    /// - empty: default rules only (excludes opt-in rules like `stale-by-git`)
    /// - non-empty: only the listed rules, drawn from default and `extra_rules`
    ///
    /// `extra_rules` lets the caller register opt-in rules (e.g. `StaleByGitRule`)
    /// whose construction depends on resources the usecase does not own
    /// (git command, project path, current time).
    pub fn execute(
        &self,
        scraps: &[Scrap],
        rule_names: &[LintRuleName],
        extra_rules: Vec<Box<dyn LintRule>>,
    ) -> ScrapsResult<Vec<LintWarning>> {
        let backlinks_map = BacklinksMap::new(scraps);
        let tags = Tags::new(&scraps);

        let mut rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(DeadEndRule),
            Box::new(LonelyRule),
            Box::new(SelfLinkRule),
            Box::new(OverlinkingRule),
            Box::new(BrokenLinkRule),
            Box::new(BrokenHeadingRefRule),
        ];

        if !rule_names.is_empty() {
            rules.extend(extra_rules);
            rules.retain(|r| rule_names.contains(&r.name()));
        }

        let warnings: Vec<LintWarning> = rules
            .par_iter()
            .flat_map(|rule| rule.check(&scraps, &backlinks_map, &tags))
            .collect();

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_run_and_aggregate() {
        // v1: each scrap below triggers a different rule.
        // - no_links: dead_end (no outbound) and lonely (no inbound)
        // - self_linker: self_link
        // - overlinker: overlinking (duplicate refs to no_links)
        // - linker_to_unknown: broken_link ([[unknown]] doesn't resolve)
        // - heading_referrer: broken_heading_ref ([[no_links#missing]] - target
        //   exists but heading doesn't)
        let scraps = vec![
            Scrap::new("no_links", &None, "plain text"),
            Scrap::new("self_linker", &None, "[[self_linker]] [[no_links]]"),
            Scrap::new("overlinker", &None, "[[no_links]] [[no_links]]"),
            Scrap::new("linker_to_unknown", &None, "[[unknown]]"),
            Scrap::new("heading_referrer", &None, "[[no_links#missing]]"),
        ];

        let usecase = LintUsecase::new();
        let warnings = usecase.execute(&scraps, &[], Vec::new()).unwrap();

        let rule_names: Vec<&LintRuleName> = warnings.iter().map(|w| &w.rule_name).collect();
        assert!(rule_names.contains(&&LintRuleName::DeadEnd));
        assert!(rule_names.contains(&&LintRuleName::Lonely));
        assert!(rule_names.contains(&&LintRuleName::SelfLink));
        assert!(rule_names.contains(&&LintRuleName::Overlinking));
        assert!(rule_names.contains(&&LintRuleName::BrokenLink));
        assert!(rule_names.contains(&&LintRuleName::BrokenHeadingRef));
    }

    #[test]
    fn clean_project_no_warnings() {
        // v1: explicit `#[[]]` tags don't introduce broken links, and mutual
        // `[[]]` refs satisfy lonely / dead_end.
        let scraps = vec![
            Scrap::new("a", &None, "[[b]] #[[shared_tag]]"),
            Scrap::new("b", &None, "[[a]] #[[shared_tag]]"),
        ];

        let usecase = LintUsecase::new();
        let warnings = usecase.execute(&scraps, &[], Vec::new()).unwrap();

        assert!(warnings.is_empty());
    }

    #[test]
    fn empty_project_no_errors() {
        let usecase = LintUsecase::new();
        let warnings = usecase.execute(&[], &[], Vec::new()).unwrap();

        assert!(warnings.is_empty());
    }

    #[test]
    fn filter_by_specific_rule() {
        let scraps = vec![
            Scrap::new("no_links", &None, "plain text"),
            Scrap::new("self_linker", &None, "[[self_linker]] [[no_links]]"),
        ];

        let usecase = LintUsecase::new();
        let warnings = usecase
            .execute(&scraps, &[LintRuleName::DeadEnd], Vec::new())
            .unwrap();

        assert!(warnings
            .iter()
            .all(|w| w.rule_name == LintRuleName::DeadEnd));
        assert!(!warnings.is_empty());
    }

    #[test]
    fn default_excludes_opt_in_rules() {
        // StaleByGit must not run when no rule is explicitly requested,
        // even if an extra rule is registered.
        use crate::usecase::lint::rules::stale_by_git::StaleByGitRule;
        use scraps_libs::git::tests::GitCommandTest;
        use std::path::PathBuf;

        let scraps = vec![
            Scrap::new("a", &None, "[[b]]"),
            Scrap::new("b", &None, "[[a]]"),
        ];
        let stale_rule = StaleByGitRule {
            git_command: GitCommandTest::new(),
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 1,
            now_ts: 1_700_000_000,
        };

        let usecase = LintUsecase::new();
        let warnings = usecase
            .execute(&scraps, &[], vec![Box::new(stale_rule)])
            .unwrap();

        assert!(warnings
            .iter()
            .all(|w| w.rule_name != LintRuleName::StaleByGit));
    }

    #[test]
    fn extra_rule_runs_when_explicitly_selected() {
        use crate::usecase::lint::rules::stale_by_git::StaleByGitRule;
        use scraps_libs::git::tests::GitCommandTest;
        use std::path::PathBuf;

        // GitCommandTest returns ts=0 for every scrap, so any positive
        // threshold flags everything as stale.
        let scraps = vec![
            Scrap::new("a", &None, "[[b]]"),
            Scrap::new("b", &None, "[[a]]"),
        ];
        let stale_rule = StaleByGitRule {
            git_command: GitCommandTest::new(),
            scraps_dir: PathBuf::from("/tmp"),
            threshold_days: 1,
            now_ts: 1_700_000_000,
        };

        let usecase = LintUsecase::new();
        let warnings = usecase
            .execute(
                &scraps,
                &[LintRuleName::StaleByGit],
                vec![Box::new(stale_rule)],
            )
            .unwrap();

        assert!(!warnings.is_empty());
        assert!(warnings
            .iter()
            .all(|w| w.rule_name == LintRuleName::StaleByGit));
    }
}
