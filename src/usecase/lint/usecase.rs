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

    pub fn execute(
        &self,
        scraps: &[Scrap],
        rule_names: &[LintRuleName],
    ) -> ScrapsResult<Vec<LintWarning>> {
        let backlinks_map = BacklinksMap::new(scraps);
        let tags = Tags::new(&scraps);

        let all_rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(DeadEndRule),
            Box::new(LonelyRule),
            Box::new(SelfLinkRule),
            Box::new(OverlinkingRule),
            Box::new(BrokenLinkRule),
            Box::new(BrokenHeadingRefRule),
        ];

        let rules: Vec<Box<dyn LintRule>> = if rule_names.is_empty() {
            all_rules
        } else {
            all_rules
                .into_iter()
                .filter(|r| rule_names.contains(&r.name()))
                .collect()
        };

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
        let warnings = usecase.execute(&scraps, &[]).unwrap();

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
        let warnings = usecase.execute(&scraps, &[]).unwrap();

        assert!(warnings.is_empty());
    }

    #[test]
    fn empty_project_no_errors() {
        let usecase = LintUsecase::new();
        let warnings = usecase.execute(&[], &[]).unwrap();

        assert!(warnings.is_empty());
    }

    #[test]
    fn filter_by_specific_rule() {
        let scraps = vec![
            Scrap::new("no_links", &None, "plain text"),
            Scrap::new("self_linker", &None, "[[self_linker]] [[no_links]]"),
        ];

        let usecase = LintUsecase::new();
        let warnings = usecase.execute(&scraps, &[LintRuleName::DeadEnd]).unwrap();

        assert!(warnings
            .iter()
            .all(|w| w.rule_name == LintRuleName::DeadEnd));
        assert!(!warnings.is_empty());
    }
}
