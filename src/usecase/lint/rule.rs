use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::usecase::build::model::backlinks_map::BacklinksMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LintRuleName {
    DeadEnd,
    Lonely,
    SelfLink,
    Overlinking,
    BrokenLink,
    BrokenHeadingRef,
}

impl LintRuleName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::DeadEnd => "dead-end",
            Self::Lonely => "lonely",
            Self::SelfLink => "self-link",
            Self::Overlinking => "overlinking",
            Self::BrokenLink => "broken-link",
            Self::BrokenHeadingRef => "broken-heading-ref",
        }
    }
}

pub struct LintWarning {
    pub rule_name: LintRuleName,
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
    fn name(&self) -> LintRuleName;
    fn check(
        &self,
        scraps: &[Scrap],
        backlinks_map: &BacklinksMap,
        tags: &Tags,
    ) -> Vec<LintWarning>;
}
