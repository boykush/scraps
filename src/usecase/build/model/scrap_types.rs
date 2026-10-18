use std::collections::BTreeMap;

use scraps_libs::model::scrap::Scrap;

/// One type in use: its name as written and how many scraps carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrapType {
    pub name: String,
    pub scraps_count: usize,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(title: &str, scrap_type: &str) -> Scrap {
        Scrap::new(title, &None, &format!("---\ntype: \"{scrap_type}\"\n---\n"))
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

        assert_eq!(types.iter().count(), 2);
        assert_eq!(types.get("How-to").unwrap().scraps_count, 1);
        assert_eq!(types.get("Reference").unwrap().scraps_count, 2);
        assert_eq!(types.get("reference"), None);
    }

    #[test]
    fn it_is_empty_when_no_scrap_has_a_type() {
        let scraps = [Scrap::new("a", &None, "#[[Blog]]")];

        assert_eq!(ScrapTypes::new(&scraps).iter().count(), 0);
    }
}
