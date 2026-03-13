use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintWarning};

pub struct SingletonTagRule;

impl LintRule for SingletonTagRule {
    fn name(&self) -> &str {
        "singleton-tag"
    }

    fn check(
        &self,
        _scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        tags: &Tags,
    ) -> Vec<LintWarning> {
        tags.iter()
            .filter(|tag| {
                let backlinks = backlinks_map.get(&tag.title().clone().into());
                backlinks.len() == 1
            })
            .map(|tag| LintWarning {
                rule_name: self.name().to_string(),
                scrap_path: tag.title().to_string(),
                message: "tag is referenced by only 1 scrap".to_string(),
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
    fn detect_singleton_tag() {
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]]");
        let scraps = vec![scrap1];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, "singleton-tag");
        assert_eq!(warnings[0].scrap_path, "tag1");
    }

    #[test]
    fn skip_tag_with_multiple_backlinks() {
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]]");
        let scrap2 = Scrap::new("scrap2", &None, "[[tag1]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_multiple_singleton_tags() {
        let scrap1 = Scrap::new("scrap1", &None, "[[tag1]] [[tag2]]");
        let scraps = vec![scrap1];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn skip_scrap_link_not_tag() {
        // scrap2 links to scrap1 — scrap1 is a scrap, not a tag
        let scrap1 = Scrap::new("scrap1", &None, "some text");
        let scrap2 = Scrap::new("scrap2", &None, "[[scrap1]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        // Tags only includes links that don't resolve to existing scraps
        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }
}
