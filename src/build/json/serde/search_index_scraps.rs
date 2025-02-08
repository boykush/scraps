use itertools::Itertools;
use scraps_libs::model::scrap::Scrap;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeSearchIndexScrap {
    title: String,
    slug: String,
}

impl SerializeSearchIndexScrap {
    pub fn new(scrap: &Scrap) -> SerializeSearchIndexScrap {
        SerializeSearchIndexScrap {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeSearchIndexScraps(Vec<SerializeSearchIndexScrap>);

impl SerializeSearchIndexScraps {
    pub fn new(scraps: &[Scrap]) -> SerializeSearchIndexScraps {
        let serialize_scraps = scraps.iter().map(SerializeSearchIndexScrap::new);

        SerializeSearchIndexScraps(serialize_scraps.collect_vec())
    }
}
