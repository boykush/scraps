use itertools::Itertools;
use scraps_libs::model::{file::ScrapFileStem, scrap::Scrap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeSearchIndexScrap {
    title: String,
    file_stem: String,
}

impl SerializeSearchIndexScrap {
    fn new(scrap: &Scrap) -> SerializeSearchIndexScrap {
        SerializeSearchIndexScrap {
            title: scrap.title.to_string(),
            file_stem: ScrapFileStem::from(scrap.self_link()).to_string(),
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
