use itertools::Itertools;
use url::Url;

use scraps_libs::model::{file::ScrapFileStem, scrap::Scrap};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeLinkScrap {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    html_content: String,
    thumbnail: Option<Url>,
}

impl SerializeLinkScrap {
    fn new(scrap: &Scrap) -> SerializeLinkScrap {
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_link().clone()));
        SerializeLinkScrap {
            ctx: scrap.ctx.as_ref().map(|c| c.to_string()),
            title: scrap.title.to_string(),
            html_file_name,
            html_content: scrap.content.to_string(),
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
