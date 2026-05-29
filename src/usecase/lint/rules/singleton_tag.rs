use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::lint::rule::{scrap_relative_path, LintRule, LintRuleName, LintWarning};

/// Detect `#[[tag]]` tags that are used by only a single scrap.
///
/// A tag is meant to be a concept shared across two or more scraps. A tag that
/// resolves to just one scrap is usually a mistake: the author likely meant a
/// regular `[[link]]` and prefixed it with `#` by accident. This rule flags
/// each explicitly-declared tag whose backlinks resolve to exactly one scrap.
///
/// Hierarchical tags are judged per exact tag: `#[[ai/ml]]` and `#[[ai/db]]`
/// are each singletons even though they share the `ai` ancestor, because
/// `get_tag` counts a tag together with its descendants but not its siblings.
/// A parent tag that has descendants in other scraps (e.g. `#[[ai]]` while
/// another scrap has `#[[ai/ml]]`) is therefore not a singleton.
pub struct SingletonTagRule;

impl LintRule for SingletonTagRule {
    fn name(&self) -> LintRuleName {
        LintRuleName::SingletonTag
    }

    fn check(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        _tags: &Tags,
    ) -> Vec<LintWarning> {
        scraps
            .iter()
            .flat_map(|scrap| {
                let path = scrap_relative_path(scrap);
                let md_text = scrap.md_text();
                scrap
                    .tags()
                    .iter()
                    .filter(|tag| backlinks_map.get_tag(tag).len() == 1)
                    .map(move |tag| {
                        let display = tag.to_string();
                        LintWarning {
                            rule_name: LintRuleName::SingletonTag,
                            scrap_path: path.clone(),
                            message: format!(
                                "tag #[[{display}]] is used by only one scrap (a tag should connect two or more scraps; did you mean a link [[{display}]]?)"
                            ),
                            source: Some(md_text.to_string()),
                            span: find_tag_span(md_text, &display),
                        }
                    })
            })
            .collect()
    }
}

/// Best-effort byte span of the first `#[[tag]]` occurrence in the scrap body,
/// used to anchor the rendered warning snippet. Returns `None` when the literal
/// form can't be located (e.g. unusual whitespace inside the brackets).
fn find_tag_span(text: &str, display: &str) -> Option<(usize, usize)> {
    let pattern = format!("#[[{display}]]");
    text.find(&pattern)
        .map(|start| (start, start + pattern.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_singleton_tag() {
        let scrap = Scrap::new("a", &None, "text #[[solo]] more");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule_name, LintRuleName::SingletonTag);
        assert_eq!(warnings[0].scrap_path, "a.md");
        assert!(warnings[0].message.contains("solo"));
        assert!(warnings[0].span.is_some());
    }

    #[test]
    fn skip_shared_tag() {
        let scrap1 = Scrap::new("a", &None, "#[[shared]]");
        let scrap2 = Scrap::new("b", &None, "#[[shared]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }

    #[test]
    fn flag_only_the_singleton_among_mixed_tags() {
        let scrap1 = Scrap::new("a", &None, "#[[shared]] #[[only_a]]");
        let scrap2 = Scrap::new("b", &None, "#[[shared]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "a.md");
        assert!(warnings[0].message.contains("only_a"));
    }

    #[test]
    fn hierarchical_siblings_are_each_singletons() {
        // Strict semantics: `ai/ml` and `ai/db` each resolve to one scrap, so
        // both are flagged even though the `ai` ancestor connects two scraps.
        let scrap1 = Scrap::new("a", &None, "#[[ai/ml]]");
        let scrap2 = Scrap::new("b", &None, "#[[ai/db]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 2);
        assert!(warnings.iter().any(|w| w.message.contains("ai/ml")));
        assert!(warnings.iter().any(|w| w.message.contains("ai/db")));
    }

    #[test]
    fn parent_with_descendant_in_another_scrap_is_not_singleton() {
        // `get_tag` aggregates descendants: scrap `a` declares `project/alpha`
        // and scrap `b` declares its descendant `project/alpha/sub`, so
        // `project/alpha` resolves to two scraps and is not flagged. Only the
        // leaf `project/alpha/sub` (one scrap) is a singleton.
        let scrap1 = Scrap::new("a", &None, "#[[project/alpha]]");
        let scrap2 = Scrap::new("b", &None, "#[[project/alpha/sub]]");
        let scraps = vec![scrap1, scrap2];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].scrap_path, "b.md");
        assert!(warnings[0].message.contains("project/alpha/sub"));
    }

    #[test]
    fn same_tag_repeated_in_one_scrap_is_a_single_warning() {
        // Tags are deduped within a scrap, so a tag repeated in the same scrap
        // still resolves to one scrap and yields exactly one warning.
        let scrap = Scrap::new("a", &None, "#[[dup]] and again #[[dup]]");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn skip_scrap_without_tags() {
        let scrap = Scrap::new("a", &None, "[[link]] plain text, no tags");
        let scraps = vec![scrap];
        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let warnings = SingletonTagRule.check(&scraps, &backlinks_map, &tags);
        assert!(warnings.is_empty());
    }
}
