use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use crate::usecase::scrap::scrap_ref::{ScrapRef, find_scrap};
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;

/// Result for scrap backlinks lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupScrapBacklinksResult {
    pub scrap: ScrapRef,
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
        key: &ScrapKey,
    ) -> ScrapsResult<Vec<LookupScrapBacklinksResult>> {
        // Verify the target scrap exists
        find_scrap(scraps, key)?;

        // Use BacklinksMap to find all scraps that link to the target
        let backlinks_map = BacklinksMap::new(scraps);
        let linking_scraps = backlinks_map.get(key);

        // Convert each linking scrap to LookupScrapBacklinksResult
        let results: Vec<LookupScrapBacklinksResult> = linking_scraps
            .iter()
            .map(|linking_scrap| LookupScrapBacklinksResult {
                scrap: (*linking_scrap).into(),
                md_text: linking_scrap.md_text().to_string(),
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
            .execute(&scraps, &ScrapKey::from_path_str("target_scrap"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        // Check that we got the expected linking scraps
        let titles: Vec<String> = results.iter().map(|r| r.scrap.title.clone()).collect();
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
                &Some("Context".into()),
                "# Target Scrap\n\nContent of target scrap.",
            ),
        ];

        let usecase = LookupScrapBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &ScrapKey::from_path_str("Context/target_scrap"))
            .expect("Should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].scrap.title, "scrap1");
    }

    #[test]
    fn test_lookup_scrap_backlinks_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = LookupScrapBacklinksUsecase::new();

        let result = usecase.execute(&scraps, &ScrapKey::from_path_str("Nonexistent Scrap"));

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
            .execute(&scraps, &ScrapKey::from_path_str("target_scrap"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
