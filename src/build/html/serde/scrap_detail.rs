use url::Url;

use crate::build::model::scrap_with_commited_ts::ScrapWithCommitedTs;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct SerializeScrapDetail {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
}

impl SerializeScrapDetail {
    pub fn new(scrap_with_commited_ts: &ScrapWithCommitedTs) -> SerializeScrapDetail {
        let scrap = scrap_with_commited_ts.scrap();
        let commited_ts = scrap_with_commited_ts.commited_ts();
        SerializeScrapDetail {
            title: scrap.title.to_string(),
            slug: scrap.title.slug.to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
        }
    }
}
