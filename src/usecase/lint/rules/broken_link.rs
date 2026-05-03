use std::collections::HashSet;

use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

/// Detect `[[wikilink]]` references that don't resolve to any existing scrap.
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
                scrap
                    .links()
                    .iter()
                    .filter(|link| !scrap_self_keys.contains(link))
                    .map(move |link| LintWarning {
                        rule_name: LintRuleName::BrokenLink,
                        scrap_path: path.clone(),
                        message: format!("broken wikilink: [[{}]]", link),
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
