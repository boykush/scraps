use scraps_libs::model::file::ScrapFileStem;
use url::Url;

use crate::usecase::build::model::scrap_with_commited_ts::ScrapWithCommitedTs;

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
pub struct ScrapDetailTera {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    html_content: String,
    thumbnail: Option<Url>,
    commited_ts: Option<i64>,
}

impl From<&ScrapWithCommitedTs> for ScrapDetailTera {
    fn from(scrap_with_commited_ts: &ScrapWithCommitedTs) -> Self {
        let scrap = scrap_with_commited_ts.scrap();
        let commited_ts = scrap_with_commited_ts.commited_ts();
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_link()));
        ScrapDetailTera {
            ctx: scrap.ctx.as_ref().map(|ctx| ctx.to_string()),
            title: scrap.title.to_string(),
            html_file_name,
            html_content: scrap.content.to_string(),
            thumbnail: scrap.thumbnail.clone(),
            commited_ts,
        }
    }
}
