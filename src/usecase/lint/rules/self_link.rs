use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

pub struct SelfLinkRule;

impl LintRule for SelfLinkRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::SelfLink
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .filter(|scrap| scrap.links().contains(&scrap.self_key()))
            .map(|scrap| LintWarning {
                rule_name: self.name(),
                scrap_path: scrap_relative_path(scrap),
                message: "scrap links to itself".to_string(),
                source: Some(scrap.md_text().to_string()),
                span: find_self_link_span(scrap),
            })
            .collect()
    }
}

fn find_self_link_span(scrap: &Scrap) -> Option<(usize, usize)> {
    let self_key_str = scrap.self_key().to_string();
    let pattern = format!("[[{}]]", self_key_str);
    let text = scrap.md_text();
    text.find(&pattern)
        .map(|start| (start, start + pattern.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_self_link() {
        let scrap = Scrap::new("myself", &None, "text [[myself]] more");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SelfLinkRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::SelfLink);
        assert_eq!(warnings[0].scrap_path, "myself.md");
        assert!(warnings[0].span.is_some());
    }

    #[test]
    fn detect_self_link_with_context() {
        let scrap = Scrap::new(
            "title",
            &Some("Context".into()),
            "text [[Context/title]] more",
        );
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SelfLinkRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "Context/title.md");
    }

    #[test]
    fn skip_links_to_other_scraps() {
        let scrap = Scrap::new("a", &None, "[[b]] [[c]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SelfLinkRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn self_link_mixed_with_other_links() {
        let scrap = Scrap::new("me", &None, "[[other]] [[me]] [[another]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SelfLinkRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
    }
}
