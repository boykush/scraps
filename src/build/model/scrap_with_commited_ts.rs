use itertools::Itertools;

use crate::libs::model::scrap::Scrap;

#[derive(Clone)]
pub struct ScrapWithCommitedTs {
    v: Scrap,
    commited_ts: Option<i64>,
}

impl ScrapWithCommitedTs {
    pub fn new(scrap: &Scrap, commited_ts: &Option<i64>) -> ScrapWithCommitedTs {
        ScrapWithCommitedTs {
            v: scrap.to_owned(),
            commited_ts: commited_ts.to_owned(),
        }
    }

    pub fn scrap(&self) -> Scrap {
        self.v.clone()
    }

    pub fn commited_ts(&self) -> Option<i64> {
        self.commited_ts.clone()
    }
}

pub struct ScrapsWithCommitedTs(Vec<ScrapWithCommitedTs>);

impl ScrapsWithCommitedTs {
    pub fn new(scraps: &Vec<ScrapWithCommitedTs>) -> ScrapsWithCommitedTs {
        ScrapsWithCommitedTs(scraps.to_owned())
    }

    pub fn to_vec(&self) -> Vec<ScrapWithCommitedTs> {
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
