use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::collections::HashMap;
use std::path::PathBuf;

/// Result for scrap links lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapLinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupScrapLinksUsecase {
    scraps_dir_path: PathBuf,
}

impl LookupScrapLinksUsecase {
    pub fn new(scraps_dir_path: &PathBuf) -> LookupScrapLinksUsecase {
        LookupScrapLinksUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(
        &self,
        title: &Title,
        ctx: &Option<Ctx>,
    ) -> ScrapsResult<Vec<LookupScrapLinksResult>> {
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

        // Find the target scrap
        let target_scrap = scraps
            .iter()
            .find(|scrap| scrap.self_key() == target_key)
            .ok_or_else(|| {
                anyhow::anyhow!("Scrap not found: title='{}', ctx='{:?}'", title, ctx)
            })?;

        // Create scrap key-to-scrap mapping for efficient lookup
        let scrap_map: HashMap<ScrapKey, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key(), scrap))
            .collect();

        // Convert each link to LookupScrapLinksResult
        let results: Vec<LookupScrapLinksResult> = target_scrap
            .links()
            .iter()
            .filter_map(|link_key| {
                // Find the linked scrap
                scrap_map.get(link_key).map(|linked_scrap| {
                    let scrap_key = &linked_scrap.self_key();
                    let title: Title = scrap_key.into();
                    let ctx: Option<Ctx> = scrap_key.into();

                    LookupScrapLinksResult {
                        title,
                        ctx,
                        md_text: linked_scrap.md_text().to_string(),
                    }
                })
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
    fn test_lookup_scrap_links_success() {
        let test_resource_path =
            PathBuf::from("tests/resource/scrap/lookup_links/test_lookup_scrap_links_success");
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");
        let md_path_2 = scraps_dir_path.join("scrap2.md");
        let md_path_3 = scraps_dir_path.join("scrap3.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(
                &md_path_1,
                b"# Scrap 1\n\nThis links to [[scrap2]] and [[scrap3]].",
            )
            .add_file(&md_path_2, b"# Scrap 2\n\nContent of scrap 2.")
            .add_file(&md_path_3, b"# Scrap 3\n\nContent of scrap 3.");

        resources.run(|| {
            let usecase = LookupScrapLinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("scrap1"), &None)
                .expect("Should succeed");

            assert_eq!(results.len(), 2);

            // Check that we got the expected linked scraps
            let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
            assert!(titles.contains(&"scrap2".to_string()));
            assert!(titles.contains(&"scrap3".to_string()));
        });
    }

    #[test]
    fn test_lookup_scrap_links_with_context() {
        let test_resource_path =
            PathBuf::from("tests/resource/scrap/lookup_links/test_lookup_scrap_links_with_context");
        let scraps_dir_path = test_resource_path.join("scraps");
        let context_dir_path = scraps_dir_path.join("Context");

        let md_path_1 = context_dir_path.join("scrap1.md");
        let md_path_2 = scraps_dir_path.join("scrap2.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&context_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nThis links to [[scrap2]].")
            .add_file(&md_path_2, b"# Scrap 2\n\nContent of scrap 2.");

        resources.run(|| {
            let usecase = LookupScrapLinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("scrap1"), &Some(Ctx::from("Context")))
                .expect("Should succeed");

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].title.to_string(), "scrap2");
        });
    }

    #[test]
    fn test_lookup_scrap_links_not_found() {
        let test_resource_path =
            PathBuf::from("tests/resource/scrap/lookup_links/test_lookup_scrap_links_not_found");
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nContent.");

        resources.run(|| {
            let usecase = LookupScrapLinksUsecase::new(&scraps_dir_path);

            let result = usecase.execute(&Title::from("Nonexistent Scrap"), &None);

            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Scrap not found"));
        });
    }

    #[test]
    fn test_lookup_scrap_links_no_links() {
        let test_resource_path =
            PathBuf::from("tests/resource/scrap/lookup_links/test_lookup_scrap_links_no_links");
        let scraps_dir_path = test_resource_path.join("scraps");

        let md_path_1 = scraps_dir_path.join("scrap1.md");

        let mut resources = TestResources::new();
        resources
            .add_dir(&scraps_dir_path)
            .add_file(&md_path_1, b"# Scrap 1\n\nThis scrap has no links.");

        resources.run(|| {
            let usecase = LookupScrapLinksUsecase::new(&scraps_dir_path);

            let results = usecase
                .execute(&Title::from("scrap1"), &None)
                .expect("Should succeed");

            assert_eq!(results.len(), 0);
        });
    }
}
