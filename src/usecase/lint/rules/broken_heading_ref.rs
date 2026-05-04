use std::collections::{HashMap, HashSet};

use scraps_libs::{
    markdown,
    model::{key::ScrapKey, scrap::Scrap, tags::Tags},
    slugify,
};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

/// Detect `[[name#heading]]` references whose `#heading` part doesn't match any
/// heading in the target scrap.
///
/// Heading resolution (v1): the referenced heading and each candidate heading
/// in the target scrap are normalized with `slugify::by_dash` before comparison.
/// The same slugifier feeds the URL fragment that HTML rendering emits, so
/// "no warning" means the rendered `target.html#fragment` lines up with a
/// heading slug in the target document.
///
/// Missing-target-scrap is the `broken-link` rule's territory; this rule stays
/// silent in that case to keep one warning per real cause.
pub struct BrokenHeadingRefRule;

impl LintRule for BrokenHeadingRefRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::BrokenHeadingRef
    }

    fn check(
        &self,
        scraps: &[Scrap],
        _backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        let scrap_by_key: HashMap<ScrapKey, &Scrap> =
            scraps.iter().map(|s| (s.self_key(), s)).collect();

        let heading_slugs_cache: HashMap<ScrapKey, HashSet<String>> = scrap_by_key
            .iter()
            .map(|(key, scrap)| {
                let slugs = markdown::query::headings(scrap.md_text())
                    .into_iter()
                    .map(|h| slugify::by_dash(&h.text))
                    .collect();
                (key.clone(), slugs)
            })
            .collect();

        let mut warnings = Vec::new();
        for scrap in scraps {
            let path = scrap_relative_path(scrap);
            for link in markdown::query::wikilinks(scrap.md_text()) {
                let Some(heading) = link.heading.as_ref() else {
                    continue;
                };
                let target_key = ScrapKey::from(&link);
                let Some(target_slugs) = heading_slugs_cache.get(&target_key) else {
                    continue;
                };
                let ref_slug = slugify::by_dash(heading);
                if ref_slug.is_empty() || target_slugs.contains(&ref_slug) {
                    continue;
                }
                warnings.push(LintWarning {
                    rule_name: LintRuleName::BrokenHeadingRef,
                    scrap_path: path.clone(),
                    message: format!(
                        "broken heading reference: [[{}#{}]] (heading not found in target scrap)",
                        target_key, heading
                    ),
                    source: None,
                    span: None,
                });
            }
        }
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_broken_heading_ref() {
        let target = Scrap::new("target", &None, "## present\n\nbody\n");
        let referrer = Scrap::new("a", &None, "see [[target#missing]]");
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::BrokenHeadingRef);
        assert_eq!(warnings[0].scrap_path, "a.md");
        assert!(warnings[0].message.contains("target"));
        assert!(warnings[0].message.contains("missing"));
    }

    #[test]
    fn skip_resolved_heading_ref() {
        let target = Scrap::new("target", &None, "## present\n\nbody\n");
        let referrer = Scrap::new("a", &None, "see [[target#present]]");
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn skip_link_without_heading() {
        let target = Scrap::new("target", &None, "no headings here");
        let referrer = Scrap::new("a", &None, "see [[target]]");
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn skip_when_target_scrap_missing() {
        // Missing-target-scrap is broken-link's job. broken-heading-ref must
        // not double-report that case.
        let referrer = Scrap::new("a", &None, "see [[ghost#whatever]]");
        let scraps = vec![referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_with_alias_and_context() {
        let target = Scrap::new("Eric Evans", &Some("Person".into()), "## Bio\n\ntext\n");
        let referrer = Scrap::new("a", &None, "[[Person/Eric Evans#missing|Eric]]");
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn case_insensitive_heading_match() {
        // slugify::by_dash lowercases, so "Section" and "section" align.
        let target = Scrap::new("target", &None, "## Section\n\nbody\n");
        let referrer = Scrap::new("a", &None, "see [[target#section]]");
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn detect_self_referencing_broken_heading() {
        let scrap = Scrap::new("a", &None, "## present\n\n[[a#missing]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn detect_multiple_broken_heading_refs_within_one_scrap() {
        let target = Scrap::new("target", &None, "## one\n\nbody\n");
        let referrer = Scrap::new(
            "a",
            &None,
            "[[target#missing1]] and [[target#missing2]] and [[target#one]]",
        );
        let scraps = vec![target, referrer];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = BrokenHeadingRefRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 2);
    }
}
