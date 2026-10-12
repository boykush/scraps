use itertools::Itertools;

use scraps_libs::model::{page_path::PagePath, scrap::Scrap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeLinkScrap {
    ctx: Option<String>,
    title: String,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
}

impl SerializeLinkScrap {
    fn new(scrap: &Scrap) -> SerializeLinkScrap {
        let url_path = PagePath::of_scrap(&scrap.self_key()).to_url_path();
        SerializeLinkScrap {
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
            title: scrap.stem().to_string(),
            url_path,
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct LinkScrapsTera(Vec<SerializeLinkScrap>);

impl LinkScrapsTera {
    pub fn new(scraps: &[&Scrap]) -> LinkScrapsTera {
        let serialize_scraps = scraps
            .iter()
            .map(|scrap| SerializeLinkScrap::new(scrap))
            .collect_vec();

        LinkScrapsTera(serialize_scraps)
    }
}
