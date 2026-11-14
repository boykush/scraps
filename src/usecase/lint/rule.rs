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
    UnlinkableKey,
    StaleByGit,
    UnportableKey,
    SourceHasPage,
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
            Self::UnlinkableKey => "unlinkable-key",
            Self::StaleByGit => "stale-by-git",
            Self::UnportableKey => "unportable-key",
            Self::SourceHasPage => "source-has-page",
        }
    }

    /// Rules selected when `scraps lint` is invoked without `--rule`.
    /// Excludes the opt-in rules: each checks a policy the wiki chooses and is
    /// enabled through its `[lint.*]` config section.
    pub fn default_rules() -> Vec<LintRuleName> {
        vec![
            Self::DeadEnd,
            Self::Lonely,
            Self::SelfLink,
            Self::Overlinking,
            Self::BrokenLink,
            Self::BrokenHeadingRef,
            Self::UnlinkableKey,
            Self::SourceHasPage,
        ]
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
        Some(ctx) => format!("{}/{}.md", ctx, scrap.stem()),
        None => format!("{}.md", scrap.stem()),
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
