use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::path::PathBuf;

/// Result for scrap backlinks lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapBacklinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupScrapBacklinksUsecase {
    scraps_dir_path: PathBuf,
}

impl LookupScrapBacklinksUsecase {
    pub fn new(scraps_dir_path: &PathBuf) -> LookupScrapBacklinksUsecase {
        LookupScrapBacklinksUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(
        &self,
        title: &Title,
        ctx: &Option<Ctx>,
    ) -> ScrapsResult<Vec<LookupScrapBacklinksResult>> {
        // Load all scraps from directory
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Create scrap key for the target scrap
        let target_key = if let Some(ctx) = ctx {
            ScrapKey::with_ctx(title, ctx)
        } else {
            ScrapKey::from(title.clone())
        };

        // Verify the target scrap exists
        let _target_scrap = scraps
            .iter()
            .find(|scrap| scrap.self_key() == target_key)
            .ok_or_else(|| {
                anyhow::anyhow!("Scrap not found: title='{}', ctx='{:?}'", title, ctx)
            })?;

        // Use BacklinksMap to find all scraps that link to the target
        let backlinks_map = BacklinksMap::new(&scraps);
        let linking_scraps = backlinks_map.get(&target_key);

        // Convert each linking scrap to LookupScrapBacklinksResult
        let results: Vec<LookupScrapBacklinksResult> = linking_scraps
            .into_iter()
            .map(|linking_scrap| {
                let scrap_key = &linking_scrap.self_key();
                let title: Title = scrap_key.into();
                let ctx: Option<Ctx> = scrap_key.into();

                LookupScrapBacklinksResult {
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
    use scraps_libs::tests::TestResources;
    use std::path::PathBuf;

    #[test]
    fn test_lookup_scrap_backlinks_success() {
        let test_resource_path = PathBuf::from(
            "tests/resource/scrap/lookup_backlinks/test_lookup_scrap_backlinks_success",
        );
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");
        let md_path_2 = scraps_dir_path.join("scrap2.md");
        let md_path_3 = scraps_dir_path.join("target_scrap.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nThis links to [[target_scrap]].")
            .add_file(
                &md_path_2,
                b"# Scrap 2\n\nThis also links to [[target_scrap]].",
            )
            .add_file(&md_path_3, b"# Target Scrap\n\nContent of target scrap.");

        resources.run(|| {
            let usecase = LookupScrapBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("target_scrap"), &None)
                .expect("Should succeed");

            assert_eq!(results.len(), 2);

            // Check that we got the expected linking scraps
            let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
            assert!(titles.contains(&"scrap1".to_string()));
            assert!(titles.contains(&"scrap2".to_string()));
        });
    }

    #[test]
    fn test_lookup_scrap_backlinks_with_context() {
        let test_resource_path = PathBuf::from(
            "tests/resource/scrap/lookup_backlinks/test_lookup_scrap_backlinks_with_context",
        );
        let scraps_dir_path = test_resource_path.join("scraps");
        let context_dir_path = scraps_dir_path.join("Context");

        let md_path_1 = scraps_dir_path.join("scrap1.md");
        let md_path_2 = context_dir_path.join("target_scrap.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&context_dir_path)
            .add_file(
                &md_path_1,
                b"# Scrap 1\n\nThis links to [[Context/target_scrap]].",
            )
            .add_file(&md_path_2, b"# Target Scrap\n\nContent of target scrap.");

        resources.run(|| {
            let usecase = LookupScrapBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("target_scrap"), &Some(Ctx::from("Context")))
                .expect("Should succeed");

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].title.to_string(), "scrap1");
        });
    }

    #[test]
    fn test_lookup_scrap_backlinks_not_found() {
        let test_resource_path = PathBuf::from(
            "tests/resource/scrap/lookup_backlinks/test_lookup_scrap_backlinks_not_found",
        );
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nContent.");

        resources.run(|| {
            let usecase = LookupScrapBacklinksUsecase::new(&scraps_dir_path);

            let result = usecase.execute(&Title::from("Nonexistent Scrap"), &None);

            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Scrap not found"));
        });
    }

    #[test]
    fn test_lookup_scrap_backlinks_no_backlinks() {
        let test_resource_path = PathBuf::from(
            "tests/resource/scrap/lookup_backlinks/test_lookup_scrap_backlinks_no_backlinks",
        );
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("target_scrap.md");

        let mut resources = TestResources::new();
        resources.add_dir(&scraps_dir_path).add_file(
            &md_path_1,
            b"# Target Scrap\n\nThis scrap has no backlinks.",
        );

        resources.run(|| {
            let usecase = LookupScrapBacklinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("target_scrap"), &None)
                .expect("Should succeed");

            assert_eq!(results.len(), 0);
        });
    }
}
