use scraps_libs::model::page_path::PagePath;
use url::Url;

use crate::usecase::build::model::scrap_detail::ScrapDetail;

use super::content::ContentTera;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct ScrapDetailTera<'a> {
    ctx: Option<String>,
    title: String,
    /// Page URL relative to `base_url`, already percent-encoded.
    url_path: String,
    content: ContentTera<'a>,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
}

impl<'a> From<&'a ScrapDetail> for ScrapDetailTera<'a> {
    fn from(scrap_detail: &'a ScrapDetail) -> Self {
        let scrap = scrap_detail.scrap();
        let commited_ts = scrap_detail.commited_ts();
        let content = scrap_detail.content();
        let url_path = PagePath::of_scrap(&scrap.self_key()).to_url_path();
        ScrapDetailTera {
            ctx: scrap.ctx().as_ref().map(|ctx| ctx.to_string()),
            title: scrap.stem().to_string(),
            url_path,
            content: content.into(),
            thumbnail: scrap.thumbnail(),
            commited_ts,
        }
    }
}
