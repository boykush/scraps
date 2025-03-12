use itertools::Itertools;
use url::Url;

use scraps_libs::model::{scrap::Scrap, slug::Slug};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeLinkScrap {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
}

impl SerializeLinkScrap {
    fn new(scrap: &Scrap) -> SerializeLinkScrap {
        SerializeLinkScrap {
            title: scrap.title.to_string(),
            slug: Slug::from(scrap.title.clone()).to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct LinkScrapsTera(Vec<SerializeLinkScrap>);

impl LinkScrapsTera {
    pub fn new(scraps: &[Scrap]) -> LinkScrapsTera {
        let serialize_scraps = scraps.iter().map(SerializeLinkScrap::new).collect_vec();

        LinkScrapsTera(serialize_scraps)
    }
}
