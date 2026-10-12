use std::collections::{BTreeMap, HashSet};

use scraps_libs::{model::scrap::Scrap, slugify};

/// Slug for a type whose name slugifies to nothing, such as `-`.
const EMPTY_SLUG: &str = "type";

/// One type in use: its name as written, the directory under `types/` that
/// holds the views narrowed to it, and how many scraps carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapType {
    pub name: String,
    pub slug: String,
    pub scraps_count: usize,
}

/// The types in use across a wiki, in name order. Built once per build: a
/// type's slug depends on the other types, so every page must read it here.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScrapTypes(Vec<ScrapType>);

impl ScrapTypes {
    pub fn new(scraps: &[Scrap]) -> ScrapTypes {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for name in scraps.iter().filter_map(Scrap::scrap_type) {
            *counts.entry(name).or_default() += 1;
        }

        // Types are case-sensitive and slugs are not, so `Blog` and `blog`
        // want the same path. Name order decides who keeps it; the rest get a
        // numeric suffix, which keeps the paths the same on every build.
        let mut taken: HashSet<String> = HashSet::new();
        let types = counts
            .into_iter()
            .map(|(name, scraps_count)| {
                let slug = free_slug(name, &taken);
                taken.insert(slug.clone());
                ScrapType {
                    name: name.to_string(),
                    slug,
                    scraps_count,
                }
            })
            .collect();

        ScrapTypes(types)
    }

    pub fn get(&self, name: &str) -> Option<&ScrapType> {
        self.0
            .binary_search_by(|scrap_type| scrap_type.name.as_str().cmp(name))
            .ok()
            .map(|index| &self.0[index])
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScrapType> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn free_slug(name: &str, taken: &HashSet<String>) -> String {
    let base = match slugify::by_dash(name) {
        slug if slug.is_empty() => EMPTY_SLUG.to_string(),
        slug => slug,
    };

    let mut slug = base.clone();
    let mut suffix = 2;
    while taken.contains(&slug) {
        slug = format!("{base}-{suffix}");
        suffix += 1;
    }
    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(title: &str, scrap_type: &str) -> Scrap {
        Scrap::new(title, &None, &format!("---\ntype: \"{scrap_type}\"\n---\n"))
    }

    fn slugs(types: &ScrapTypes) -> Vec<(&str, &str)> {
        types
            .iter()
            .map(|scrap_type| (scrap_type.name.as_str(), scrap_type.slug.as_str()))
            .collect()
    }

    #[test]
    fn it_counts_the_scraps_of_each_type_in_name_order() {
        let scraps = [
            typed("a", "Reference"),
            typed("b", "How-to"),
            typed("c", "Reference"),
            Scrap::new("untyped", &None, "#[[Reference]]"),
        ];

        let types = ScrapTypes::new(&scraps);

        assert_eq!(types.len(), 2);
        assert_eq!(types.get("How-to").unwrap().scraps_count, 1);
        assert_eq!(types.get("Reference").unwrap().scraps_count, 2);
        assert_eq!(types.get("reference"), None);
    }

    #[test]
    fn it_is_empty_when_no_scrap_has_a_type() {
        let scraps = [Scrap::new("a", &None, "#[[Blog]]")];

        assert!(ScrapTypes::new(&scraps).is_empty());
    }

    #[test]
    fn it_slugifies_a_type_as_one_segment() {
        let scraps = [typed("a", "BigQuery Table"), typed("b", "ADR/Accepted")];

        assert_eq!(
            slugs(&ScrapTypes::new(&scraps)),
            vec![
                ("ADR/Accepted", "adr-slash-accepted"),
                ("BigQuery Table", "bigquery-table"),
            ]
        );
    }

    #[test]
    fn it_keeps_types_whose_slugs_collide_on_separate_paths() {
        // `blog-2` is already a type's own slug, so the suffix has to move on.
        let scraps = [typed("a", "blog"), typed("b", "Blog"), typed("c", "Blog 2")];

        assert_eq!(
            slugs(&ScrapTypes::new(&scraps)),
            vec![("Blog", "blog"), ("Blog 2", "blog-2"), ("blog", "blog-3")]
        );
    }

    #[test]
    fn it_gives_a_type_with_an_empty_slug_a_path() {
        let scraps = [typed("a", "-")];

        assert_eq!(slugs(&ScrapTypes::new(&scraps)), vec![("-", "type")]);
    }
}
