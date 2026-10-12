use std::collections::HashSet;

use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning, scrap_relative_path};

/// Detect `[[wikilink]]` references, and Markdown links to a `.md` path,
/// that don't resolve to any existing scrap.
///
/// Pre-v1, unresolved links were silently turned into "implicit tags" via
/// `Tags::new`. v1 keeps tags and scrap-links as separate namespaces:
/// `#[[tag]]` is a tag, `[[name]]` is a scrap link. An unresolved scrap link
/// is therefore a real bug (typo or stale reference) that this rule surfaces.
pub struct BrokenLinkRule;

impl LintRule for BrokenLinkRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::BrokenLink
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let scrap_self_keys: HashSet<ScrapKey> =
            scraps.iter().map(|scrap| scrap.self_key()).collect();

        scraps
            .iter()
            .flat_map(|scrap| {
                let path = scrap_relative_path(scrap);
                // One warning per distinct broken link, as the author wrote it.
                let mut seen = HashSet::new();
                scrap
                    .link_occurrences()
                    .filter(|link| {
                        link.target
                            .as_ref()
                            .is_none_or(|key| !scrap_self_keys.contains(key))
                    })
                    .filter_map(|link| match (link.markdown, link.target) {
                        (Some(written), _) => Some(format!("broken link: {written}")),
                        (None, Some(key)) => Some(format!("broken wikilink: [[{key}]]")),
                        (None, None) => None,
                    })
                    .filter(|message| seen.insert(message.clone()))
                    .map(|message| LintWarning {
                        rule_name: LintRuleName::BrokenLink,
                        scrap_path: path.clone(),
                        message,
                        source: None,
                        span: None,
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
    fn detect_unresolved_wikilink() {
        let scrap = Scrap::new("a", &None, "[[nonexistent]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::BrokenLink);
        assert_eq!(warnings[0].scrap_path, "a.md");
        assert!(warnings[0].message.contains("nonexistent"));
    }

    #[test]
    fn skip_resolved_wikilink() {
        let scrap1 = Scrap::new("a", &None, "");
        let scrap2 = Scrap::new("b", &None, "[[a]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_multiple_broken_links_within_one_scrap() {
        let scrap = Scrap::new("a", &None, "[[broken1]] and [[broken2]] and [[a]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        // [[a]] resolves (self), broken1 and broken2 don't.
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn skip_explicit_tag_not_treated_as_link() {
        // `#[[ai]]` is a tag, not a wikilink. It should not show up as broken.
        let scrap = Scrap::new("a", &None, "#[[ai]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_markdown_link_to_a_missing_scrap() {
        let scraps = vec![
            Scrap::new(
                "orders",
                &Some("tables".into()),
                "[c](./customers.md) [gone](./gone.md) [gone](./gone.md) [out](../../out.md)",
            ),
            Scrap::new("customers", &Some("tables".into()), ""),
        ];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        let messages: Vec<&str> = warnings.iter().map(|w| w.message.as_str()).collect();
        assert_eq!(
            messages,
            vec![
                "broken link: [gone](./gone.md)",
                "broken link: [out](../../out.md)"
            ]
        );
        assert_eq!(warnings[0].scrap_path, "tables/orders.md");
    }

    #[test]
    fn skip_markdown_link_that_is_not_a_scrap_link() {
        let scrap = Scrap::new("a", &None, "[w](https://example.com/x.md) [h](/x.html)");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        assert!(
            BrokenLinkRule
                .check(&scraps, &backlinks_map, &tags)
                .is_empty()
        );
    }

    #[test]
    fn detect_unresolved_with_context() {
        let scrap = Scrap::new("foo", &None, "[[Programming/missing]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenLinkRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("Programming/missing"));
    }
}
