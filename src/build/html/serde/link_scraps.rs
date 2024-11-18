use itertools::Itertools;
use url::Url;

use crate::libs::model::scrap::Scrap;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeLinkScrap {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
}

impl SerializeLinkScrap {
    pub fn new(scrap: &Scrap) -> SerializeLinkScrap {
        SerializeLinkScrap {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct SerializeLinkScraps(Vec<SerializeLinkScrap>);

impl SerializeLinkScraps {
    pub fn new(scraps: &[Scrap]) -> SerializeLinkScraps {
        let serialize_scraps = scraps.iter().map(SerializeLinkScrap::new).collect_vec();

        SerializeLinkScraps(serialize_scraps)
    }
}
