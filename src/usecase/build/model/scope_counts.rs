use std::collections::{BTreeMap, HashMap};

use scraps_libs::model::{scrap::Scrap, tag::Tag, tags::Tags};

use super::backlinks_map::BacklinksMap;

/// How many scraps a scope holds, in all and per type.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize)]
pub struct TypeCounts {
    pub total: usize,
    pub by_type: BTreeMap<String, usize>,
}

impl TypeCounts {
    fn of<'a>(scraps: impl IntoIterator<Item = &'a Scrap>) -> TypeCounts {
        let mut counts = TypeCounts::default();
        for scrap in scraps {
            counts.total += 1;
            if let Some(name) = scrap.scrap_type() {
                *counts.by_type.entry(name.to_string()).or_default() += 1;
            }
        }
        counts
    }
}

/// The counts behind every scope a reader can pick: the whole wiki and each
/// tag, a tag counting the scraps of its descendants too.
#[derive(Debug, Default)]
pub struct ScopeCounts {
    all: TypeCounts,
    by_tag: HashMap<Tag, TypeCounts>,
}

impl ScopeCounts {
    pub fn new(scraps: &[Scrap], backlinks_map: &BacklinksMap) -> ScopeCounts {
        let by_tag = Tags::new(scraps)
            .into_iter()
            .map(|tag| {
                let counts = TypeCounts::of(backlinks_map.get_tag(&tag).iter().copied());
                (tag, counts)
            })
            .collect();
        ScopeCounts {
            all: TypeCounts::of(scraps),
            by_tag,
        }
    }

    /// The counts of one tag, or of the whole wiki for `None`.
    pub fn of(&self, tag: Option<&Tag>) -> &TypeCounts {
        static EMPTY: std::sync::LazyLock<TypeCounts> =
            std::sync::LazyLock::new(TypeCounts::default);
        match tag {
            Some(tag) => self.by_tag.get(tag).unwrap_or(&EMPTY),
            None => &self.all,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scrap(title: &str, scrap_type: Option<&str>, body: &str) -> Scrap {
        let front = scrap_type.map_or(String::new(), |t| format!("---\ntype: {t}\n---\n"));
        Scrap::new(title, &None, &format!("{front}{body}"))
    }

    #[test]
    fn it_counts_each_scope_in_all_and_per_type() {
        let scraps = [
            scrap("a", Some("Book"), "#[[Security/Auth]]"),
            scrap("b", Some("Blog"), "#[[Security]]"),
            scrap("c", Some("Book"), "#[[Agile]]"),
            scrap("d", None, "#[[Security/Auth]]"),
        ];
        let backlinks_map = BacklinksMap::new(&scraps);
        let counts = ScopeCounts::new(&scraps, &backlinks_map);

        let all = counts.of(None);
        assert_eq!(all.total, 4);
        assert_eq!(all.by_type.get("Book"), Some(&2));

        // A parent tag counts what its children carry.
        let security = counts.of(Some(&Tag::from("Security")));
        assert_eq!(security.total, 3);
        assert_eq!(security.by_type.get("Book"), Some(&1));
        assert_eq!(security.by_type.get("Blog"), Some(&1));

        let auth = counts.of(Some(&Tag::from("Security/Auth")));
        assert_eq!(auth.total, 2);
        assert_eq!(auth.by_type.get("Blog"), None);
    }

    #[test]
    fn it_counts_nothing_for_an_unknown_tag() {
        let counts = ScopeCounts::new(&[], &BacklinksMap::new(&[]));
        assert_eq!(counts.of(Some(&Tag::from("none"))).total, 0);
    }
}
