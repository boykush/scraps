use itertools::Itertools;
use url::Url;

use scraps_libs::{
    markdown,
    model::{base_url::BaseUrl, file::ScrapFileStem, scrap::Scrap},
};

#[derive(serde::Serialize, Clone, PartialEq, Debug)]
struct SerializeLinkScrap {
    ctx: Option<String>,
    title: String,
    html_file_name: String,
    html_text: String,
    thumbnail: Option<Url>,
}

impl SerializeLinkScrap {
    fn new(scrap: &Scrap, base_url: &BaseUrl) -> SerializeLinkScrap {
        let content = markdown::convert::to_content(scrap.md_text(), base_url);
        let html_file_name = format!("{}.html", ScrapFileStem::from(scrap.self_key().clone()));
        SerializeLinkScrap {
            ctx: scrap.ctx().as_ref().map(|c| c.to_string()),
            title: scrap.title().to_string(),
            html_file_name,
            html_text: content.to_string(),
            thumbnail: scrap.thumbnail(),
        }
    }
}

#[derive(serde::Serialize, PartialEq, Debug)]
pub struct LinkScrapsTera(Vec<SerializeLinkScrap>);

impl LinkScrapsTera {
    pub fn new(scraps: &[Scrap], base_url: &BaseUrl) -> LinkScrapsTera {
        let serialize_scraps = scraps
            .iter()
            .map(|s| SerializeLinkScrap::new(s, base_url))
            .collect_vec();

        LinkScrapsTera(serialize_scraps)
    }
}
