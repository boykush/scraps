use std::collections::BTreeMap;

use scraps_libs::model::{page_path::PagePath, scrap::Scrap};

/// One type in use: its name as written and how many scraps carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapType {
    pub name: String,
    pub scraps_count: usize,
}

impl ScrapType {
    /// The directory that holds the views narrowed to this type.
    pub fn page_path(&self) -> PagePath {
        PagePath::of_type(&self.name)
    }
}

/// The types in use across a wiki, in name order. Types compare as written,
/// case included, so `Blog` and `blog` are two types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScrapTypes(Vec<ScrapType>);

impl ScrapTypes {
    pub fn new(scraps: &[Scrap]) -> ScrapTypes {
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        for name in scraps.iter().filter_map(Scrap::scrap_type) {
            *counts.entry(name).or_default() += 1;
        }

        let types = counts
            .into_iter()
            .map(|(name, scraps_count)| ScrapType {
                name: name.to_string(),
                scraps_count,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(title: &str, scrap_type: &str) -> Scrap {
        Scrap::new(title, &None, &format!("---\ntype: \"{scrap_type}\"\n---\n"))
    }

    fn paths(types: &ScrapTypes) -> Vec<(&str, String)> {
        types
            .iter()
            .map(|scrap_type| (scrap_type.name.as_str(), scrap_type.page_path().to_string()))
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
    fn it_puts_each_type_under_its_name_as_written() {
        let scraps = [
            typed("a", "blog"),
            typed("b", "Blog"),
            typed("c", "Blog 2"),
            typed("d", "ADR/Accepted"),
            typed("e", "-"),
        ];

        assert_eq!(
            paths(&ScrapTypes::new(&scraps)),
            vec![
                ("-", "types/-/".to_string()),
                ("ADR/Accepted", "types/ADR%2FAccepted/".to_string()),
                ("Blog", "types/Blog/".to_string()),
                ("Blog 2", "types/Blog 2/".to_string()),
                ("blog", "types/blog/".to_string()),
            ]
        );
    }
}
