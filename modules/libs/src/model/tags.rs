use std::collections::HashSet;

use super::{scrap::Scrap, tag::Tag};

/// The set of tags that exist in a wiki, aggregated from explicit `#[[tag]]`
/// occurrences across all scraps. Hierarchical tags are auto-aggregated:
/// a `#[[a/b/c]]` occurrence implicitly creates `a/b` and `a` as well
/// (Logseq-style).
#[derive(PartialEq, Debug, Clone)]
pub struct Tags(HashSet<Tag>);

impl IntoIterator for Tags {
    type Item = Tag;
    type IntoIter = std::collections::hash_set::IntoIter<Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Tags {
    /// Build the tag set by aggregating each scrap's explicit `#[[tag]]`
    /// declarations and folding in their proper ancestors for hierarchical
    /// tags.
    pub fn new(scraps: &[Scrap]) -> Tags {
        let mut all = HashSet::new();
        for scrap in scraps {
            for tag in scrap.tags() {
                all.insert(tag.clone());
                for ancestor in tag.ancestors() {
                    all.insert(ancestor);
                }
            }
        }
        Tags(all)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, Tag> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // v1 shape: Tags::new aggregates explicit `#[[tag]]` occurrences across
    // scraps, with Logseq-style ancestor auto-aggregation for hierarchical
    // tags. Implicit derivation from unresolved `[[]]` links is removed.

    fn collect_sorted(tags: &Tags) -> Vec<String> {
        let mut v: Vec<String> = tags.iter().map(|t| format!("{}", t)).collect();
        v.sort();
        v
    }

    #[test]
    fn it_aggregates_explicit_tags_across_scraps() {
        let scrap1 = Scrap::new("a", &None, "#[[ai]]");
        let scrap2 = Scrap::new("b", &None, "#[[programming]]");
        let tags = Tags::new(&[scrap1, scrap2]);
        assert_eq!(
            collect_sorted(&tags),
            vec!["ai".to_string(), "programming".to_string()]
        );
    }

    #[test]
    fn it_dedupes_same_tag_across_scraps() {
        let scrap1 = Scrap::new("a", &None, "#[[ai]]");
        let scrap2 = Scrap::new("b", &None, "#[[ai]]");
        let tags = Tags::new(&[scrap1, scrap2]);
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn it_auto_aggregates_ancestors_for_hierarchical_tag() {
        // #[[ai/ml/transformer]] should also produce ai/ml and ai
        let scrap = Scrap::new("a", &None, "#[[ai/ml/transformer]]");
        let tags = Tags::new(&[scrap]);
        assert_eq!(
            collect_sorted(&tags),
            vec![
                "ai".to_string(),
                "ai/ml".to_string(),
                "ai/ml/transformer".to_string(),
            ]
        );
    }

    #[test]
    fn it_does_not_derive_tags_from_unresolved_wikilinks() {
        // The pre-v1 implicit derivation from `[[name]]` is gone. Unresolved
        // wikilinks are no longer turned into tags.
        let scrap = Scrap::new("a", &None, "[[unresolved-link]] but no #[[]] here");
        let tags = Tags::new(&[scrap]);
        assert!(tags.is_empty());
    }

    #[test]
    fn it_overlapping_hierarchies_dedupe_at_aggregate_level() {
        // a/b/c and a/b/d both contribute "a" and "a/b" — should dedupe.
        let scrap1 = Scrap::new("s1", &None, "#[[a/b/c]]");
        let scrap2 = Scrap::new("s2", &None, "#[[a/b/d]]");
        let tags = Tags::new(&[scrap1, scrap2]);
        assert_eq!(
            collect_sorted(&tags),
            vec![
                "a".to_string(),
                "a/b".to_string(),
                "a/b/c".to_string(),
                "a/b/d".to_string(),
            ]
        );
    }

    #[test]
    fn it_empty_when_no_scraps() {
        let tags = Tags::new(&[]);
        assert!(tags.is_empty());
    }

    #[test]
    fn it_empty_when_scraps_have_no_tags() {
        let scrap = Scrap::new("a", &None, "plain body, no tags or links");
        let tags = Tags::new(&[scrap]);
        assert!(tags.is_empty());
    }
}
