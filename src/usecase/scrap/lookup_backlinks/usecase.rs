use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;

/// Result for scrap backlinks lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapBacklinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupScrapBacklinksUsecase;

impl LookupScrapBacklinksUsecase {
    pub fn new() -> LookupScrapBacklinksUsecase {
        LookupScrapBacklinksUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        title: &Title,
        ctx: &Option<Ctx>,
    ) -> ScrapsResult<Vec<LookupScrapBacklinksResult>> {
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
        let backlinks_map = BacklinksMap::new(scraps);
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

    #[test]
    fn test_lookup_scrap_backlinks_success() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nThis links to [[target_scrap]].",
            ),
            Scrap::new(
                "scrap2",
                &None,
                "# Scrap 2\n\nThis also links to [[target_scrap]].",
            ),
            Scrap::new(
                "target_scrap",
                &None,
                "# Target Scrap\n\nContent of target scrap.",
            ),
        ];

        let usecase = LookupScrapBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("target_scrap"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got the expected linking scraps
        let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert!(titles.contains(&"scrap1".to_string()));
        assert!(titles.contains(&"scrap2".to_string()));
    }

    #[test]
    fn test_lookup_scrap_backlinks_with_context() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nThis links to [[Context/target_scrap]].",
            ),
            Scrap::new(
                "target_scrap",
                &Some("Context"),
                "# Target Scrap\n\nContent of target scrap.",
            ),
        ];

        let usecase = LookupScrapBacklinksUsecase::new();

        let results = usecase
            .execute(
                &scraps,
                &Title::from("target_scrap"),
                &Some(Ctx::from("Context")),
            )
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "scrap1");
    }

    #[test]
    fn test_lookup_scrap_backlinks_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = LookupScrapBacklinksUsecase::new();

        let result = usecase.execute(&scraps, &Title::from("Nonexistent Scrap"), &None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }

    #[test]
    fn test_lookup_scrap_backlinks_no_backlinks() {
        let scraps = vec![Scrap::new(
            "target_scrap",
            &None,
            "# Target Scrap\n\nThis scrap has no backlinks.",
        )];

        let usecase = LookupScrapBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("target_scrap"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
