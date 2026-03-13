use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct LintWarning {
    pub rule_name: String,
    pub scrap_path: String,
    pub message: String,
    pub source: Option<String>,
    pub span: Option<(usize, usize)>,
}

pub fn scrap_relative_path(scrap: &Scrap) -> String {
    match scrap.ctx() {
        Some(ctx) => format!("{}/{}.md", ctx, scrap.title()),
        None => format!("{}.md", scrap.title()),
    }
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
