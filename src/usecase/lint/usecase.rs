use std::collections::HashSet;
use std::path::PathBuf;

use scraps_libs::markdown::extract;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;

use crate::error::ScrapsResult;
use crate::usecase::read_scraps;

pub struct LintResult {
    pub scrap_title: Title,
    pub broken_link: ScrapKey,
}

pub struct LintUsecase {
    scraps_dir_path: PathBuf,
}

impl LintUsecase {
    pub fn new(scraps_dir_path: &PathBuf) -> LintUsecase {
        LintUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(&self) -> ScrapsResult<Vec<LintResult>> {
        let paths = read_scraps::to_scrap_paths(&self.scraps_dir_path)?;

        let scraps = paths
            .iter()
            .map(|path| read_scraps::to_scrap_by_path(&self.scraps_dir_path, path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        let scrap_keys: HashSet<ScrapKey> = scraps.iter().map(|s| s.self_key()).collect();

        let warnings = scraps
            .iter()
            .flat_map(|scrap| {
                let tag_keys: HashSet<ScrapKey> =
                    extract::scrap_tags(scrap.md_text()).into_iter().collect();

                scrap
                    .links()
                    .iter()
                    .filter(|link| !scrap_keys.contains(link))
                    .filter(|link| !tag_keys.contains(link))
                    .map(|link| LintResult {
                        scrap_title: scrap.title().clone(),
                        broken_link: link.clone(),
                    })
                    .collect::<Vec<_>>()
            })
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
    fn it_no_warnings_for_existing_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_scrap("page_a.md", b"[[page_b]]")
            .add_scrap("page_b.md", b"[[page_a]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn it_warns_for_broken_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("page_a.md", b"[[non_existing]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].scrap_title, Title::from("page_a"));
        assert_eq!(
            result[0].broken_link,
            ScrapKey::from(Title::from("non_existing"))
        );
    }

    #[rstest]
    fn it_no_warnings_for_hash_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("page_a.md", b"#[[intentional_tag]] [[non_existing]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        // intentional_tag is excluded by #[[]], but non_existing is still a broken link
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].broken_link,
            ScrapKey::from(Title::from("non_existing"))
        );
    }

    #[rstest]
    fn it_no_warnings_when_all_tags_have_hash(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("page_a.md", b"#[[tag1]] #[[tag2]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn it_allows_hash_tag_with_existing_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        // #[[existing]] where existing.md exists is allowed (not an error)
        project
            .add_scrap("page_a.md", b"#[[page_b]]")
            .add_scrap("page_b.md", b"some content");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn it_warns_for_context_broken_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("page_a.md", b"[[Category/non_existing]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].broken_link,
            ScrapKey::with_ctx(&"non_existing".into(), &"Category".into())
        );
    }

    #[rstest]
    fn it_no_warnings_for_context_hash_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("page_a.md", b"#[[Category/tag1]]");

        let usecase = LintUsecase::new(&project.scraps_dir);
        let result = usecase.execute().unwrap();

        assert_eq!(result.len(), 0);
    }
}
