use itertools::Itertools;

use std::collections::HashMap;

use scraps_libs::{
    html::{self, EmbedMode},
    model::{base_url::BaseUrl, content::Content, key::ScrapKey, scrap::Scrap},
};

#[derive(Clone)]
pub struct ScrapDetail {
    v: Scrap,
    content: Content,
    commited_ts: Option<i64>,
}

impl ScrapDetail {
    pub fn new(
        scrap: &Scrap,
        commited_ts: &Option<i64>,
        base_url: &BaseUrl,
        scrap_texts: &HashMap<ScrapKey, String>,
    ) -> ScrapDetail {
        let content = html::to_content(scrap.md_text(), base_url, EmbedMode::Expand(scrap_texts));
        ScrapDetail {
            v: scrap.to_owned(),
            content,
            commited_ts: commited_ts.to_owned(),
        }
    }

    pub fn scrap(&self) -> &Scrap {
        &self.v
    }

    pub fn commited_ts(&self) -> Option<i64> {
        self.commited_ts
    }

    pub fn content(&self) -> &Content {
        &self.content
    }
}

pub struct ScrapDetails(Vec<ScrapDetail>);

impl ScrapDetails {
    pub fn new(scrap_details: Vec<ScrapDetail>) -> ScrapDetails {
        ScrapDetails(scrap_details)
    }

    pub fn as_slice(&self) -> &[ScrapDetail] {
        &self.0
    }

    pub fn to_scraps(&self) -> Vec<Scrap> {
        self.0.iter().map(|sc| sc.scrap().clone()).collect_vec()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
