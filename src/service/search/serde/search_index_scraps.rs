use itertools::Itertools;
use scraps_libs::model::{page_path::PagePath, scrap::Scrap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeSearchIndexScrap {
    /// What a wiki-link to the scrap holds: its path without `.md`.
    key: String,
    /// The name a search result shows: the title under its context.
    link_title: String,
    url_path: String,
}

impl SerializeSearchIndexScrap {
    fn new(scrap: &Scrap) -> SerializeSearchIndexScrap {
        let link_title = match scrap.ctx() {
            Some(ctx) => format!("{ctx}/{}", scrap.title()),
            None => scrap.title().to_string(),
        };
        SerializeSearchIndexScrap {
            key: scrap.self_key().to_string(),
            link_title,
            url_path: PagePath::of_scrap(&scrap.self_key()).to_url_path(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SearchIndexScrapsTera(Vec<SerializeSearchIndexScrap>);

impl SearchIndexScrapsTera {
    pub fn new(scraps: &[Scrap]) -> SearchIndexScrapsTera {
        let serialize_scraps = scraps.iter().map(SerializeSearchIndexScrap::new);

        SearchIndexScrapsTera(serialize_scraps.collect_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_names_a_scrap_by_its_title_under_its_context() {
        let scraps = [
            Scrap::new(
                "orders",
                &Some("tables".into()),
                "---\ntitle: Customer Orders\n---\n",
            ),
            Scrap::new("revenue", &Some("metrics".into()), ""),
            Scrap::new("glossary", &None, "---\ntitle: Glossary of Terms\n---\n"),
        ];

        let index = SearchIndexScrapsTera::new(&scraps);

        let rows: Vec<(&str, &str, &str)> = index
            .0
            .iter()
            .map(|s| (s.link_title.as_str(), s.key.as_str(), s.url_path.as_str()))
            .collect();
        assert_eq!(
            rows,
            vec![
                (
                    "tables/Customer Orders",
                    "tables/orders",
                    "scraps/tables/orders.html"
                ),
                (
                    "metrics/revenue",
                    "metrics/revenue",
                    "scraps/metrics/revenue.html"
                ),
                ("Glossary of Terms", "glossary", "scraps/glossary.html"),
            ]
        );
    }
}
