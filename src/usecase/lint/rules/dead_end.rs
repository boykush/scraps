use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

pub struct DeadEndRule;

impl LintRule for DeadEndRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::DeadEnd
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .filter(|scrap| scrap.links().is_empty())
            .map(|scrap| LintWarning {
                rule_name: self.name(),
                scrap_path: scrap_relative_path(scrap),
                message: "scrap has no links to other scraps".to_string(),
                source: None,
                span: None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_scrap_with_no_links() {
        let scrap = Scrap::new("orphan", &None, "no links here");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = DeadEndRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::DeadEnd);
        assert_eq!(warnings[0].scrap_path, "orphan.md");
    }

    #[test]
    fn skip_scrap_with_links() {
        let scrap = Scrap::new("linked", &None, "[[other]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = DeadEndRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_multiple_dead_ends() {
        let scrap1 = Scrap::new("orphan1", &None, "no links");
        let scrap2 = Scrap::new("orphan2", &None, "also no links");
        let scrap3 = Scrap::new("linked", &None, "[[something]]");
        let scraps = vec![scrap1, scrap2, scrap3];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = DeadEndRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn detect_dead_end_with_context() {
        let scrap = Scrap::new("contextual", &Some("Book".into()), "plain text");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = DeadEndRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "Book/contextual.md");
    }
}
