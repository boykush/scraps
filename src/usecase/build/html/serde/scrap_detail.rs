use scraps_libs::model::slug::Slug;
use url::Url;

use crate::usecase::build::model::scrap_with_commited_ts::ScrapWithCommitedTs;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct ScrapDetailTera {
    title: String,
    slug: String,
    html_content: String,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
}

impl From<&ScrapWithCommitedTs> for ScrapDetailTera {
    fn from(scrap_with_commited_ts: &ScrapWithCommitedTs) -> Self {
        let scrap = scrap_with_commited_ts.scrap();
        let commited_ts = scrap_with_commited_ts.commited_ts();
        ScrapDetailTera {
            title: scrap.title.to_string(),
            slug: Slug::from(scrap.title.clone()).to_string(),
            html_content: scrap.html_content.clone(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
        }
    }
}
