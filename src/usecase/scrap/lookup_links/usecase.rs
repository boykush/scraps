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
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn test_lookup_scrap_links_success(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_scrap(
                "scrap1.md",
                b"# Scrap 1\n\nThis links to [[scrap2]] and [[scrap3]].",
            )
            .add_scrap("scrap2.md", b"# Scrap 2\n\nContent of scrap 2.")
            .add_scrap("scrap3.md", b"# Scrap 3\n\nContent of scrap 3.");

        let usecase = LookupScrapLinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got the expected linked scraps
        let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert!(titles.contains(&"scrap2".to_string()));
        assert!(titles.contains(&"scrap3".to_string()));
    }

    #[rstest]
    fn test_lookup_scrap_links_with_context(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_scrap_with_context(
                "Context",
                "scrap1.md",
                b"# Scrap 1\n\nThis links to [[scrap2]].",
            )
            .add_scrap("scrap2.md", b"# Scrap 2\n\nContent of scrap 2.");

        let usecase = LookupScrapLinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("scrap1"), &Some(Ctx::from("Context")))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "scrap2");
    }

    #[rstest]
    fn test_lookup_scrap_links_not_found(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("scrap1.md", b"# Scrap 1\n\nContent.");

        let usecase = LookupScrapLinksUsecase::new(&project.scraps_dir);

        let result = usecase.execute(&Title::from("Nonexistent Scrap"), &None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }

    #[rstest]
    fn test_lookup_scrap_links_no_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("scrap1.md", b"# Scrap 1\n\nThis scrap has no links.");

        let usecase = LookupScrapLinksUsecase::new(&project.scraps_dir);

        let results = usecase
            .execute(&Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
