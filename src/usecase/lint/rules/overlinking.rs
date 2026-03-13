use std::collections::HashMap;

use scraps_libs::markdown::extract::scrap_links_with_duplicates;
use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintWarning};

pub struct OverlinkingRule;

impl LintRule for OverlinkingRule {
    fn name(&self) -> &str {
        "overlinking"
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .flat_map(|scrap| {
                let all_links = scrap_links_with_duplicates(scrap.md_text());
                let mut counts: HashMap<ScrapKey, usize> = HashMap::new();
                for link in &all_links {
                    *counts.entry(link.clone()).or_insert(0) += 1;
                }

                counts
                    .into_iter()
                    .filter(|(_, count)| *count > 1)
                    .map(|(key, count)| {
                        let pattern = format!("[[{}]]", key);
                        let source = scrap.md_text().to_string();
                        let span = source.find(&pattern).map(|s| (s, s + pattern.len()));

                        LintWarning {
                            rule_name: self.name().to_string(),
                            scrap_title: scrap.self_key().to_string(),
                            message: format!("link [[{}]] appears {} times", key, count),
                            source: Some(source),
                            span,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_duplicate_link() {
        let scrap = Scrap::new("test", &None, "[[a]] text [[a]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, "overlinking");
        assert!(warnings[0].message.contains("2 times"));
    }

    #[test]
    fn detect_triple_link_as_one_warning() {
        let scrap = Scrap::new("test", &None, "[[a]] [[a]] [[a]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("3 times"));
    }

    #[test]
    fn alias_and_plain_same_link() {
        // [[A|B]] resolves to ScrapKey("A"), same as [[A]]
        let scrap = Scrap::new(
            "test",
            &None,
            "[[Domain Driven Design|DDD]] [[Domain Driven Design]]",
        );
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("2 times"));
    }

    #[test]
    fn skip_all_unique_links() {
        let scrap = Scrap::new("test", &None, "[[a]] [[b]] [[c]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }
}
