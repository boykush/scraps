use itertools::Itertools;

use crate::usecase::build::model::{
    scope_counts::TypeCounts,
    scrap_types::{ScrapType, ScrapTypes},
};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct TypeTera {
    /// The type as written in frontmatter.
    pub name: String,
    /// The query that narrows a list to the type, e.g. `type=How-to`.
    query: String,
    pub scraps_count: usize,
}

impl From<&ScrapType> for TypeTera {
    fn from(scrap_type: &ScrapType) -> Self {
        TypeTera {
            name: scrap_type.name.clone(),
            query: url::form_urlencoded::Serializer::new(String::new())
                .append_pair("type", &scrap_type.name)
                .finish(),
            scraps_count: scrap_type.scraps_count,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct TypesTera(Vec<TypeTera>);

impl TypesTera {
    /// Most used first, like tags; the name breaks ties so the sidebar's
    /// top-N slice is the same on every build.
    pub fn new(types: &ScrapTypes) -> TypesTera {
        let sorted = types.iter().map(TypeTera::from).sorted_by(|a, b| {
            b.scraps_count
                .cmp(&a.scraps_count)
                .then_with(|| a.name.cmp(&b.name))
        });
        TypesTera(sorted.collect_vec())
    }

    /// The same types in the same order, each counted within one scope
    /// rather than across the wiki.
    pub fn counted_in(types: &ScrapTypes, counts: &TypeCounts) -> TypesTera {
        let TypesTera(types) = Self::new(types);
        let counted = types.into_iter().map(|scrap_type| TypeTera {
            scraps_count: counts.by_type.get(&scrap_type.name).copied().unwrap_or(0),
            ..scrap_type
        });
        TypesTera(counted.collect_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::model::scrap::Scrap;

    fn typed(title: &str, scrap_type: &str) -> Scrap {
        Scrap::new(title, &None, &format!("---\ntype: {scrap_type}\n---\n"))
    }

    #[test]
    fn it_orders_by_scraps_count_desc_then_name_asc() {
        let scraps = [
            typed("a", "Tutorial"),
            typed("b", "Reference"),
            typed("c", "Reference"),
            typed("d", "Explanation"),
        ];

        let types = TypesTera::new(&ScrapTypes::new(&scraps));

        let names: Vec<&str> = types.0.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, vec!["Reference", "Explanation", "Tutorial"]);
    }
}
