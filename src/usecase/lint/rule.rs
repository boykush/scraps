use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;

pub struct LintWarning {
    pub rule_name: String,
    pub scrap_path: String,
    pub message: String,
    pub source: Option<String>,
    pub span: Option<(usize, usize)>,
}

impl LintWarning {
    pub fn line_col(&self) -> Option<(usize, usize)> {
        let (source, (start, _)) = self.source.as_ref().zip(self.span)?;
        let before = &source[..start];
        let line = before.matches('\n').count() + 1;
        let col = start - before.rfind('\n').map_or(0, |i| i + 1) + 1;
        Some((line, col))
    }
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
