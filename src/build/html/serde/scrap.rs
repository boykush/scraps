use url::Url;

use crate::build::model::{linked_scraps_map::LinkedScrapsMap, scrap::Scrap, title::Title};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrap {
    title: String,
    slug: String,
    links: Vec<String>,
    html_content: String,
    thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
    pub linked_count: usize,
}

impl SerializeScrap {
    pub fn new(scrap: &Scrap, linked_scraps_map: &LinkedScrapsMap) -> SerializeScrap {
        let linked_count = linked_scraps_map.linked_by(&scrap.title).len();
        SerializeScrap {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
            links: scrap.links.iter().map(Title::to_string).collect(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts: scrap.commited_ts,
            linked_count,
        }
    }
}
