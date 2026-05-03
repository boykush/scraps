use std::collections::{HashMap, HashSet};

use scraps_libs::model::{key::ScrapKey, scrap::Scrap, tag::Tag};

/// Backlinks aggregated across a wiki. Two distinct namespaces are tracked:
/// scrap-to-scrap links (keyed by `ScrapKey`) and explicit `#[[tag]]`
/// occurrences (keyed by `Tag`). Tag backlinks include ancestor auto-
/// aggregation: a scrap tagged `#[[a/b/c]]` appears in the backlinks of
/// `a/b` and `a` as well.
#[derive(PartialEq, Debug)]
pub struct BacklinksMap {
    scrap_backlinks: HashMap<ScrapKey, Vec<Scrap>>,
    tag_backlinks: HashMap<Tag, Vec<Scrap>>,
}

impl BacklinksMap {
    pub fn new(scraps: &[Scrap]) -> BacklinksMap {
        BacklinksMap {
            scrap_backlinks: Self::gen_scrap_backlinks(scraps),
            tag_backlinks: Self::gen_tag_backlinks(scraps),
        }
    }

    /// Backlinks that come from `[[wikilink]]` references between scraps.
    pub fn get(&self, key: &ScrapKey) -> Vec<Scrap> {
        self.scrap_backlinks
            .get(key)
            .map_or_else(Vec::new, Vec::clone)
    }

    /// Backlinks that come from `#[[tag]]` declarations. Includes scraps
    /// tagged at any descendant level via Logseq-style auto-aggregation.
    pub fn get_tag(&self, tag: &Tag) -> Vec<Scrap> {
        self.tag_backlinks
            .get(tag)
            .map_or_else(Vec::new, Vec::clone)
    }

    fn gen_scrap_backlinks(scraps: &[Scrap]) -> HashMap<ScrapKey, Vec<Scrap>> {
        scraps.iter().fold(HashMap::new(), |mut acc, scrap| {
            for key in scrap.links() {
                acc.entry(key.clone()).or_default().push(scrap.to_owned());
            }
            acc
        })
    }

    fn gen_tag_backlinks(scraps: &[Scrap]) -> HashMap<Tag, Vec<Scrap>> {
        // For each scrap, collect the union of its declared tags and their
        // ancestors (deduped at the scrap level so a scrap tagged with both
        // `#[[a]]` and `#[[a/b]]` is only listed once under `a`).
        scraps.iter().fold(HashMap::new(), |mut acc, scrap| {
            let mut keys: HashSet<Tag> = HashSet::new();
            for tag in scrap.tags() {
                keys.insert(tag.clone());
                for ancestor in tag.ancestors() {
                    keys.insert(ancestor);
                }
            }
            for tag in keys {
                acc.entry(tag).or_default().push(scrap.to_owned());
            }
            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use scraps_libs::model::tag::Tag;
    use scraps_libs::model::title::Title;

    use super::*;

    // v1 shape: scrap-link backlinks no longer include unresolved `[[]]`
    // pretending to be tags. Explicit `#[[tag]]` occurrences populate a
    // separate tag-keyed backlinks map accessible via `get_tag`.

    #[test]
    fn it_get_returns_scrap_link_backlinks_only() {
        let scrap1 = Scrap::new("scrap1", &None, "");
        let scrap2 = Scrap::new("scrap2", &None, "[[scrap1]]");
        let scraps = vec![scrap1.clone(), scrap2.clone()];

        let backlinks_map = BacklinksMap::new(&scraps);
        assert_eq!(
            backlinks_map.get(&Title::from("scrap1").into()),
            vec![scrap2]
        );
    }

    #[test]
    fn it_get_with_context() {
        let scrap1 = Scrap::new("scrap1", &Some("Context".into()), "");
        let scrap2 = Scrap::new("scrap2", &Some("Context".into()), "[[Context/scrap1]]");
        let scrap3 = Scrap::new("scrap3", &None, "[[Context/scrap1]][[Context/scrap2]]");
        let scraps = vec![scrap1.clone(), scrap2.clone(), scrap3.clone()];

        let backlinks_map = BacklinksMap::new(&scraps);
        assert_eq!(
            backlinks_map.get(&ScrapKey::with_ctx(&"scrap1".into(), &"Context".into())),
            vec![scrap2.clone(), scrap3.clone()]
        );
        assert_eq!(
            backlinks_map.get(&ScrapKey::with_ctx(&"scrap2".into(), &"Context".into())),
            vec![scrap3.clone()]
        );
        assert_eq!(backlinks_map.get(&Title::from("scrap3").into()), vec![]);
    }

    #[test]
    fn it_get_tag_collects_explicit_tagged_scraps() {
        let scrap1 = Scrap::new("a", &None, "#[[ai]]");
        let scrap2 = Scrap::new("b", &None, "#[[ai]]");
        let scrap3 = Scrap::new("c", &None, "no tags");
        let scraps = vec![scrap1.clone(), scrap2.clone(), scrap3];

        let backlinks_map = BacklinksMap::new(&scraps);
        let mut got = backlinks_map.get_tag(&Tag::from("ai"));
        got.sort_by_key(|s| s.title().to_string());
        assert_eq!(got, vec![scrap1, scrap2]);
    }

    #[test]
    fn it_get_tag_includes_descendant_tagged_scraps_via_auto_aggregation() {
        // A scrap tagged #[[ai/ml/transformer]] should also be in backlinks
        // of `ai/ml` and `ai`.
        let scrap = Scrap::new("paper", &None, "#[[ai/ml/transformer]]");
        let scraps = vec![scrap.clone()];

        let backlinks_map = BacklinksMap::new(&scraps);
        assert_eq!(
            backlinks_map.get_tag(&Tag::from("ai/ml/transformer")),
            vec![scrap.clone()]
        );
        assert_eq!(
            backlinks_map.get_tag(&Tag::from("ai/ml")),
            vec![scrap.clone()]
        );
        assert_eq!(backlinks_map.get_tag(&Tag::from("ai")), vec![scrap]);
    }

    #[test]
    fn it_get_tag_unrelated_returns_empty() {
        let scrap = Scrap::new("a", &None, "#[[ai]]");
        let backlinks_map = BacklinksMap::new(&[scrap]);
        assert!(backlinks_map.get_tag(&Tag::from("unrelated")).is_empty());
    }

    #[test]
    fn it_get_tag_dedupes_when_scrap_tagged_with_overlapping_paths() {
        // A scrap tagged with both #[[ai]] and #[[ai/ml]] should appear
        // exactly once in `ai`'s backlinks (not twice from explicit + ancestor).
        let scrap = Scrap::new("multi", &None, "#[[ai]] and #[[ai/ml]]");
        let scraps = vec![scrap.clone()];

        let backlinks_map = BacklinksMap::new(&scraps);
        assert_eq!(backlinks_map.get_tag(&Tag::from("ai")), vec![scrap]);
    }
}
