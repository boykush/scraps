use std::path::{Path, PathBuf};

use rayon::prelude::*;
use scraps_libs::model::{scrap::Scrap, tags::Tags};

use crate::{
    error::ScrapsResult,
    usecase::{build::model::backlinks_map::BacklinksMap, read_scraps},
};

use super::{
    rule::{LintRule, LintWarning},
    rules::{
        dead_end::DeadEndRule, lonely::LonelyRule, overlinking::OverlinkingRule,
        self_link::SelfLinkRule, singleton_tag::SingletonTagRule,
    },
};

pub struct LintUsecase {
    scraps_dir_path: PathBuf,
}

impl LintUsecase {
    pub fn new(scraps_dir_path: &Path) -> LintUsecase {
        LintUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(&self) -> ScrapsResult<Vec<LintWarning>> {
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;

        let scraps = paths
            .iter()
            .map(|path| read_scraps::to_scrap_by_path(&self.scraps_dir_path, path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        let backlinks_map = BacklinksMap::new(&scraps);
        let tags = Tags::new(&scraps);

        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(DeadEndRule),
            Box::new(LonelyRule),
            Box::new(SelfLinkRule),
            Box::new(OverlinkingRule),
            Box::new(SingletonTagRule),
        ];

        let warnings: Vec<LintWarning> = rules
            .par_iter()
            .flat_map(|rule| rule.check(&scraps, &backlinks_map, &tags))
            .collect();

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn all_rules_run_and_aggregate(#[from(temp_scrap_project)] project: TempScrapProject) {
        // dead-end: no_links has no links
        // lonely: no_links has no backlinks
        // self-link: self_linker links to itself
        // overlinking: overlinker has duplicate link
        // singleton-tag: singleton_tag is only referenced once
        project
            .add_scrap("no_links.md", b"plain text")
            .add_scrap("self_linker.md", b"[[self_linker]] [[no_links]]")
            .add_scrap("overlinker.md", b"[[no_links]] [[no_links]]")
            .add_scrap("with_tag.md", b"[[singleton_tag]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let warnings = usecase.execute().unwrap();

        let rule_names: Vec<&str> = warnings.iter().map(|w| w.rule_name.as_str()).collect();
        assert!(rule_names.contains(&"dead-end"));
        assert!(rule_names.contains(&"lonely"));
        assert!(rule_names.contains(&"self-link"));
        assert!(rule_names.contains(&"overlinking"));
        assert!(rule_names.contains(&"singleton-tag"));
    }

    #[rstest]
    fn clean_project_no_warnings(#[from(temp_scrap_project)] project: TempScrapProject) {
        // Two scraps mutually linked, with shared tags
        project
            .add_scrap("a.md", b"[[b]] [[shared_tag]]")
            .add_scrap("b.md", b"[[a]] [[shared_tag]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let warnings = usecase.execute().unwrap();

        assert!(warnings.is_empty());
    }

    #[rstest]
    fn empty_project_no_errors(#[from(temp_scrap_project)] project: TempScrapProject) {
        let usecase = LintUsecase::new(&project.scraps_dir);
        let warnings = usecase.execute().unwrap();

        assert!(warnings.is_empty());
    }
}
