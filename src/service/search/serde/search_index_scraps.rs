use itertools::Itertools;
use scraps_libs::model::{page_path::PagePath, scrap::Scrap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeSearchIndexScrap {
    link_title: String,
    url_path: String,
}

impl SerializeSearchIndexScrap {
    fn new(scrap: &Scrap) -> SerializeSearchIndexScrap {
        SerializeSearchIndexScrap {
            link_title: scrap.self_key().to_string(),
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
