use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintWarning};

pub struct LonelyRule;

impl LintRule for LonelyRule {
    fn name(&self) -> &str {
        "lonely"
    }

    fn check(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .filter(|scrap| backlinks_map.get(&scrap.self_key()).is_empty())
            .map(|scrap| LintWarning {
                rule_name: self.name().to_string(),
                scrap_path: scrap_relative_path(scrap),
                message: "scrap is not linked from any other scrap".to_string(),
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
    fn detect_scrap_with_no_backlinks() {
        let scrap = Scrap::new("lonely", &None, "some text");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = LonelyRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, "lonely");
        assert_eq!(warnings[0].scrap_path, "lonely.md");
    }

    #[test]
    fn skip_scrap_with_backlinks() {
        let scrap1 = Scrap::new("target", &None, "some text");
        let scrap2 = Scrap::new("linker", &None, "[[target]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = LonelyRule.check(&scraps, &backlinks_map, &tags);
        // linker has no backlinks but target does
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "linker.md");
    }

    #[test]
    fn mutual_links_not_lonely() {
        let scrap1 = Scrap::new("a", &None, "[[b]]");
        let scrap2 = Scrap::new("b", &None, "[[a]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = LonelyRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }
}
