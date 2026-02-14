use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tag::Tag;
use scraps_libs::model::tags::Tags;
use scraps_libs::model::title::Title;
use std::path::{Path, PathBuf};

/// Result for tag backlinks lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupTagBacklinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupTagBacklinksUsecase {
    scraps_dir_path: PathBuf,
}

impl LookupTagBacklinksUsecase {
    pub fn new(scraps_dir_path: &Path) -> LookupTagBacklinksUsecase {
        LookupTagBacklinksUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(&self, tag_title: &Title) -> ScrapsResult<Vec<LookupTagBacklinksResult>> {
        // Load all scraps from directory
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Get valid tags and check if the requested title is actually a tag
        let valid_tags = Tags::new(&scraps);
        let requested_tag: Tag = tag_title.clone().into();

        // If the requested title is not a valid tag, return empty results
        if !valid_tags.into_iter().any(|tag| tag == requested_tag) {
            return Ok(Vec::new());
        }

        // Create tag key (tags don't have contexts, so we use ScrapKey::from)
        let tag_key = tag_title.clone().into();

        // Use BacklinksMap to find all scraps that link to the tag
        let backlinks_map = BacklinksMap::new(&scraps);
        let linking_scraps = backlinks_map.get(&tag_key);

        // Convert each linking scrap to LookupTagBacklinksResult
        let results: Vec<LookupTagBacklinksResult> = linking_scraps
            .into_iter()
            .map(|linking_scrap| {
                let scrap_key = &linking_scrap.self_key();
                let title: Title = scrap_key.into();
                let ctx: Option<Ctx> = scrap_key.into();

                LookupTagBacklinksResult {
                    title,
                    ctx,
                    md_text: linking_scrap.md_text().to_string(),
                }
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn test_lookup_tag_backlinks_success(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_scrap("scrap1.md", b"# Scrap 1\n\nThis links to [[test_tag]].")
            .add_scrap(
                "scrap2.md",
                b"# Scrap 2\n\nThis also links to [[test_tag]].",
            )
            .add_scrap("scrap3.md", b"# Scrap 3\n\nThis links to [[other_tag]].");

        let usecase = LookupTagBacklinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("test_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got the expected linking scraps
        let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert!(titles.contains(&"scrap1".to_string()));
        assert!(titles.contains(&"scrap2".to_string()));
        assert!(!titles.contains(&"scrap3".to_string()));
    }

    #[rstest]
    fn test_lookup_tag_backlinks_with_context_scraps(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_scrap("scrap1.md", b"# Scrap 1\n\nThis links to [[test_tag]].")
            .add_scrap_with_context(
                "Context",
                "scrap2.md",
                b"# Scrap 2\n\nThis also links to [[test_tag]].",
            );

        let usecase = LookupTagBacklinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("test_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got scraps from both root and context directory
        let scrap_keys: Vec<(String, Option<String>)> = results
            .iter()
            .map(|r| (r.title.to_string(), r.ctx.as_ref().map(|c| c.to_string())))
            .collect();

        assert!(scrap_keys.contains(&("scrap1".to_string(), None)));
        assert!(scrap_keys.contains(&("scrap2".to_string(), Some("Context".to_string()))));
    }

    #[rstest]
    fn test_lookup_tag_backlinks_no_backlinks(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "scrap1.md",
            b"# Scrap 1\n\nThis scrap doesn't reference any tags.",
        );

        let usecase = LookupTagBacklinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("nonexistent_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }

    #[rstest]
    fn test_lookup_tag_backlinks_invalid_tag(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_scrap("scrap1.md", b"# Scrap 1\n\nThis links to [[actual_tag]].")
            .add_scrap("scrap2.md", b"# Scrap 2\n\nThis links to [[actual_scrap]].")
            .add_scrap(
                "actual_scrap.md",
                b"# Actual Scrap\n\nThis is a regular scrap, not a tag.",
            );

        let usecase = LookupTagBacklinksUsecase::new(&project.scraps_dir);

        // Request backlinks for "actual_scrap" - this is a scrap title, not a tag
        // Even though scrap2 links to it, it should return empty because it's not a tag
        let results = usecase
            .execute(&Title::from("actual_scrap"))
            .expect("Should succeed");

        assert_eq!(
            results.len(),
            0,
            "Should return empty results for non-tag titles"
        );

        // Verify that actual tags still work
        let tag_results = usecase
            .execute(&Title::from("actual_tag"))
            .expect("Should succeed");

        assert_eq!(
            tag_results.len(),
            1,
            "Should return results for actual tags"
        );
        assert_eq!(tag_results[0].title.to_string(), "scrap1");
    }

    #[rstest]
    fn test_lookup_tag_backlinks_empty_scraps_directory(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let usecase = LookupTagBacklinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("any_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
