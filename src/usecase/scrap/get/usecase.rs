use crate::error::ScrapsResult;
use crate::usecase::scrap::scrap_ref::{ScrapRef, find_scrap};
use scraps_libs::model::key::ScrapKey;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::source::Source;
use scraps_libs::model::verification::{TrustTier, Verification};
use serde_json::Value;

/// Result for get scrap operation
#[derive(Debug, Clone, PartialEq)]
pub struct GetScrapResult {
    pub scrap: ScrapRef,
    pub md_text: String,
    pub scrap_type: Option<String>,
    pub description: Option<String>,
    pub frontmatter: Option<Value>,
    pub verifications: Vec<Verification>,
    pub trust_tier: TrustTier,
    pub sources: Vec<Source>,
}

pub struct GetScrapUsecase;

impl GetScrapUsecase {
    pub fn new() -> GetScrapUsecase {
        GetScrapUsecase
    }

    pub fn execute(&self, scraps: &[Scrap], key: &ScrapKey) -> ScrapsResult<GetScrapResult> {
        let target_scrap = find_scrap(scraps, key)?;

        Ok(GetScrapResult {
            scrap: target_scrap.into(),
            md_text: target_scrap.md_text().to_string(),
            scrap_type: target_scrap.scrap_type().map(str::to_string),
            description: target_scrap.description().map(str::to_string),
            frontmatter: target_scrap.frontmatter().cloned(),
            verifications: target_scrap.verifications(),
            trust_tier: target_scrap.trust_tier(),
            sources: target_scrap.sources(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_scrap_success() {
        let scraps = vec![Scrap::new(
            "scrap1",
            &None,
            "# Scrap 1\n\nContent of scrap 1.",
        )];

        let usecase = GetScrapUsecase::new();

        let result = usecase
            .execute(&scraps, &ScrapKey::from_path_str("scrap1"))
            .expect("Should succeed");

        assert_eq!(result.scrap.title, "scrap1");
        assert!(result.scrap.ctx().is_none());
        assert!(result.md_text.contains("Content of scrap 1"));
    }

    #[test]
    fn test_get_scrap_with_context() {
        let scraps = vec![Scrap::new(
            "scrap1",
            &Some("Context".into()),
            "# Scrap 1\n\nContent of scrap 1.",
        )];

        let usecase = GetScrapUsecase::new();

        let result = usecase
            .execute(&scraps, &ScrapKey::from_path_str("Context/scrap1"))
            .expect("Should succeed");

        assert_eq!(result.scrap.title, "scrap1");
        assert_eq!(result.scrap.key.to_string(), "Context/scrap1");
        assert!(result.md_text.contains("Content of scrap 1"));
    }

    #[test]
    fn test_get_scrap_carries_the_frontmatter_block() {
        let scraps = vec![
            Scrap::new("typed", &None, "---\ntype: Blog\n---\n\n# Typed\n"),
            Scrap::new("plain", &None, "# Plain\n"),
        ];

        let usecase = GetScrapUsecase::new();

        let typed = usecase.execute(&scraps, &ScrapKey::from_path_str("typed"));
        assert_eq!(
            typed.unwrap().frontmatter,
            Some(serde_json::json!({"type": "Blog"}))
        );

        let plain = usecase.execute(&scraps, &ScrapKey::from_path_str("plain"));
        assert_eq!(plain.unwrap().frontmatter, None);
    }

    #[test]
    fn test_get_scrap_not_found() {
        let scraps = vec![Scrap::new("scrap1", &None, "# Scrap 1\n\nContent.")];

        let usecase = GetScrapUsecase::new();

        let result = usecase.execute(&scraps, &ScrapKey::from_path_str("nonexistent"));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Scrap not found"));
    }
}
