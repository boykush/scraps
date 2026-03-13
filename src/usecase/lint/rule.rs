use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct LintWarning {
    pub rule_name: String,
    pub scrap_title: String,
    pub message: String,
    pub source: Option<String>,
    pub span: Option<(usize, usize)>,
}

pub trait LintRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        tags: &Tags,
    ) -> Vec<LintWarning>;
}
