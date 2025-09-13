use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::path::PathBuf;

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
    pub fn new(scraps_dir_path: &PathBuf) -> LookupTagBacklinksUsecase {
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
                    md_text: linking_scrap.md_text.clone(),
                }
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraps_libs::tests::TestResources;
    use std::path::PathBuf;

    #[test]
    fn test_lookup_tag_backlinks_success() {
        let test_resource_path =
            PathBuf::from("tests/resource/tag/lookup_backlinks/test_lookup_tag_backlinks_success");
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");
        let md_path_2 = scraps_dir_path.join("scrap2.md");
        let md_path_3 = scraps_dir_path.join("scrap3.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nThis links to [[test_tag]].")
            .add_file(&md_path_2, b"# Scrap 2\n\nThis also links to [[test_tag]].")
            .add_file(&md_path_3, b"# Scrap 3\n\nThis links to [[other_tag]].");

        resources.run(|| {
            let usecase = LookupTagBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("test_tag"))
                .expect("Should succeed");

            assert_eq!(results.len(), 2);

            // Check that we got the expected linking scraps
            let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
            assert!(titles.contains(&"scrap1".to_string()));
            assert!(titles.contains(&"scrap2".to_string()));
            assert!(!titles.contains(&"scrap3".to_string()));
        });
    }

    #[test]
    fn test_lookup_tag_backlinks_with_context_scraps() {
        let test_resource_path = PathBuf::from(
            "tests/resource/tag/lookup_backlinks/test_lookup_tag_backlinks_with_context_scraps",
        );
        let scraps_dir_path = test_resource_path.join("scraps");
        let context_dir_path = scraps_dir_path.join("Context");

        let md_path_1 = scraps_dir_path.join("scrap1.md");
        let md_path_2 = context_dir_path.join("scrap2.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&context_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nThis links to [[test_tag]].")
            .add_file(&md_path_2, b"# Scrap 2\n\nThis also links to [[test_tag]].");

        resources.run(|| {
            let usecase = LookupTagBacklinksUsecase::new(&scraps_dir_path);

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
        });
    }

    #[test]
    fn test_lookup_tag_backlinks_no_backlinks() {
        let test_resource_path = PathBuf::from(
            "tests/resource/tag/lookup_backlinks/test_lookup_tag_backlinks_no_backlinks",
        );
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");

        let mut resources = TestResources::new();
        resources.add_dir(&scraps_dir_path).add_file(
            &md_path_1,
            b"# Scrap 1\n\nThis scrap doesn't reference any tags.",
        );

        resources.run(|| {
            let usecase = LookupTagBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("nonexistent_tag"))
                .expect("Should succeed");

            assert_eq!(results.len(), 0);
        });
    }

    #[test]
    fn test_lookup_tag_backlinks_empty_scraps_directory() {
        let test_resource_path = PathBuf::from(
            "tests/resource/tag/lookup_backlinks/test_lookup_tag_backlinks_empty_scraps_directory",
        );
        let scraps_dir_path = test_resource_path.join("scraps");

        let mut resources = TestResources::new();
        resources.add_dir(&scraps_dir_path);

        resources.run(|| {
            let usecase = LookupTagBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("any_tag"))
                .expect("Should succeed");

            assert_eq!(results.len(), 0);
        });
    }
}
