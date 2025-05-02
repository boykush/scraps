use itertools::Itertools;

use scraps_libs::{
    markdown,
    model::{content::Content, scrap::Scrap},
};
use url::Url;

#[derive(Clone)]
pub struct ScrapDetail {
    v: Scrap,
    content: Content,
    commited_ts: Option<i64>,
}

impl ScrapDetail {
    pub fn new(scrap: &Scrap, commited_ts: &Option<i64>, base_url: &Url) -> ScrapDetail {
        let content = markdown::convert::to_content(&scrap.md_text, base_url);
        ScrapDetail {
            v: scrap.to_owned(),
            content: content.to_owned(),
            commited_ts: commited_ts.to_owned(),
        }
    }

    pub fn scrap(&self) -> Scrap {
        self.v.clone()
    }

    pub fn commited_ts(&self) -> Option<i64> {
        self.commited_ts
    }

    pub fn content(&self) -> Content {
        self.content.clone()
    }
}

pub struct ScrapDetails(Vec<ScrapDetail>);

impl ScrapDetails {
    pub fn new(scraps: &Vec<ScrapDetail>) -> ScrapDetails {
        ScrapDetails(scraps.to_owned())
    }

    pub fn to_vec(&self) -> Vec<ScrapDetail> {
        self.0.clone()
    }

    pub fn to_scraps(&self) -> Vec<Scrap> {
        self.0
            .clone()
            .into_iter()
            .map(|sc| sc.scrap())
            .collect_vec()
    }
}
