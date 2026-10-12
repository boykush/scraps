use std::collections::HashMap;

use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

pub struct OverlinkingRule;

impl LintRule for OverlinkingRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::Overlinking
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
                // Per target: how often it is linked, and the Markdown link
                // to show when no wiki-link to it exists.
                let mut order: Vec<ScrapKey> = Vec::new();
                let mut counts: HashMap<ScrapKey, (usize, Option<String>, bool)> = HashMap::new();
                for link in scrap.link_occurrences() {
                    let Some(key) = link.target else { continue };
                    let entry = counts.entry(key.clone()).or_insert_with(|| {
                        order.push(key);
                        (0, None, false)
                    });
                    entry.0 += 1;
                    match link.markdown {
                        Some(written) => {
                            entry.1.get_or_insert(written);
                        }
                        None => entry.2 = true,
                    }
                }

                order
                    .into_iter()
                    .filter_map(|key| {
                        let (count, markdown, has_wikilink) = counts.remove(&key)?;
                        (count > 1).then_some((key, count, markdown, has_wikilink))
                    })
                    .map(|(key, count, markdown, has_wikilink)| {
                        let pattern = match markdown {
                            Some(written) if !has_wikilink => written,
                            _ => format!("[[{}]]", key),
                        };
                        let source = scrap.md_text().to_string();
                        let span = source.find(&pattern).map(|s| (s, s + pattern.len()));

                        LintWarning {
                            rule_name: self.name(),
                            scrap_path: scrap_relative_path(scrap),
                            message: format!("link {} appears {} times", pattern, count),
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
        assert_eq!(warnings[0].rule_name, LintRuleName::Overlinking);
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
    fn detect_duplicate_markdown_link() {
        let scrap = Scrap::new(
            "orders",
            &Some("tables".into()),
            "see [c](./customers.md) and [c2](/tables/customers.md)",
        );
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings[0].message,
            "link [c](./customers.md) appears 2 times"
        );
        assert_eq!(warnings[0].span, Some((4, 23)));
    }

    #[test]
    fn markdown_and_wiki_link_to_the_same_scrap() {
        let scrap = Scrap::new("test", &None, "[a](./a.md) and [[a]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = OverlinkingRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "link [[a]] appears 2 times");
        assert!(warnings[0].span.is_some());
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
