use scraps_libs::model::file::ScrapFileStem;
use url::Url;

use crate::usecase::build::model::scrap_detail::ScrapDetail;

use super::content::ContentTera;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct ScrapDetailTera {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    content: ContentTera,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
}

impl From<ScrapDetail> for ScrapDetailTera {
    fn from(scrap_detail: ScrapDetail) -> Self {
        let scrap = scrap_detail.scrap();
        let commited_ts = scrap_detail.commited_ts();
        let content = scrap_detail.content();
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_key()));
        ScrapDetailTera {
            ctx: scrap.ctx().as_ref().map(|ctx| ctx.to_string()),
            title: scrap.title().to_string(),
            html_file_name,
            content: content.into(),
            thumbnail: scrap.thumbnail(),
            commited_ts,
        }
    }
}
