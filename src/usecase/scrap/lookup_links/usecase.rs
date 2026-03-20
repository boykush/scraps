use crate::error::ScrapsResult;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use std::collections::HashMap;

/// Result for scrap links lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapLinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupScrapLinksUsecase;

impl LookupScrapLinksUsecase {
    pub fn new() -> LookupScrapLinksUsecase {
        LookupScrapLinksUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        title: &Title,
        ctx: &Option<Ctx>,
    ) -> ScrapsResult<Vec<LookupScrapLinksResult>> {
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

    #[test]
    fn test_lookup_scrap_links_success() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &None,
                "# Scrap 1\n\nThis links to [[scrap2]] and [[scrap3]].",
            ),
            Scrap::new("scrap2", &None, "# Scrap 2\n\nContent of scrap 2."),
            Scrap::new("scrap3", &None, "# Scrap 3\n\nContent of scrap 3."),
        ];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got the expected linked scraps
        let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert!(titles.contains(&"scrap2".to_string()));
        assert!(titles.contains(&"scrap3".to_string()));
    }

    #[test]
    fn test_lookup_scrap_links_with_context() {
        let scraps = vec![
            Scrap::new(
                "scrap1",
                &Some("Context"),
                "# Scrap 1\n\nThis links to [[scrap2]].",
            ),
            Scrap::new("scrap2", &None, "# Scrap 2\n\nContent of scrap 2."),
        ];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("scrap1"), &Some(Ctx::from("Context")))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title.to_string(), "scrap2");
    }

    #[test]
    fn test_lookup_scrap_links_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = LookupScrapLinksUsecase::new();

        let result = usecase.execute(&scraps, &Title::from("Nonexistent Scrap"), &None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }

    #[test]
    fn test_lookup_scrap_links_no_links() {
        let scraps = vec![Scrap::new(
            "scrap1",
            &None,
            "# Scrap 1\n\nThis scrap has no links.",
        )];

        let usecase = LookupScrapLinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("scrap1"), &None)
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
