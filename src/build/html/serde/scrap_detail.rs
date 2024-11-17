use url::Url;

use crate::libs::model::scrap::Scrap;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrapDetail {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
    pub commited_ts: Option<i64>,
}

impl SerializeScrapDetail {
    pub fn new(scrap: &Scrap) -> SerializeScrapDetail {
        SerializeScrapDetail {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts: scrap.commited_ts,
        }
    }
}
