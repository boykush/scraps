use crate::error::ScrapsResult;
use crate::usecase::build::model::backlinks_map::BacklinksMap;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::tag::Tag;
use scraps_libs::model::tags::Tags;
use scraps_libs::model::title::Title;

/// Result for tag backlinks lookup operation
#[derive(Debug, Clone, PartialEq)]
pub struct LookupTagBacklinksResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub md_text: String,
}

pub struct LookupTagBacklinksUsecase;

impl LookupTagBacklinksUsecase {
    pub fn new() -> LookupTagBacklinksUsecase {
        LookupTagBacklinksUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        tag_title: &Title,
    ) -> ScrapsResult<Vec<LookupTagBacklinksResult>> {
        // Treat the input as a possibly-hierarchical tag path (e.g. "ai/ml").
        let requested_tag: Tag = Tag::from(tag_title.to_string().as_str());

        // Verify the requested tag actually exists in the wiki (with ancestor
        // auto-aggregation): if no scrap declares this tag at any descendant
        // level, return an empty result.
        let valid_tags = Tags::new(scraps);
        if !valid_tags.into_iter().any(|tag| tag == requested_tag) {
            return Ok(Vec::new());
        }

        let backlinks_map = BacklinksMap::new(scraps);
        let linking_scraps = backlinks_map.get_tag(&requested_tag);

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

    // v1: tags come from explicit `#[[tag]]` declarations.

    #[test]
    fn test_lookup_tag_backlinks_success() {
        let scraps = vec![
            Scrap::new("scrap1", &None, "# Scrap 1\n\nTagged #[[test_tag]]."),
            Scrap::new("scrap2", &None, "# Scrap 2\n\nAlso tagged #[[test_tag]]."),
            Scrap::new(
                "scrap3",
                &None,
                "# Scrap 3\n\nDifferent tag #[[other_tag]].",
            ),
        ];

        let usecase = LookupTagBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("test_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        let titles: Vec<String> = results.iter().map(|r| r.title.to_string()).collect();
        assert!(titles.contains(&"scrap1".to_string()));
        assert!(titles.contains(&"scrap2".to_string()));
        assert!(!titles.contains(&"scrap3".to_string()));
    }

    #[test]
    fn test_lookup_tag_backlinks_with_context_scraps() {
        let scraps = vec![
            Scrap::new("scrap1", &None, "# Scrap 1\n\nTagged #[[test_tag]]."),
            Scrap::new(
                "scrap2",
                &Some("Context".into()),
                "# Scrap 2\n\nAlso tagged #[[test_tag]].",
            ),
        ];

        let usecase = LookupTagBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("test_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 2);

        let scrap_keys: Vec<(String, Option<String>)> = results
            .iter()
            .map(|r| (r.title.to_string(), r.ctx.as_ref().map(|c| c.to_string())))
            .collect();

        assert!(scrap_keys.contains(&("scrap1".to_string(), None)));
        assert!(scrap_keys.contains(&("scrap2".to_string(), Some("Context".to_string()))));
    }

    #[test]
    fn test_lookup_tag_backlinks_no_backlinks() {
        let scraps = vec![Scrap::new(
            "scrap1",
            &None,
            "# Scrap 1\n\nThis scrap doesn't reference any tags.",
        )];

        let usecase = LookupTagBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("nonexistent_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_lookup_tag_backlinks_invalid_tag() {
        let scraps = vec![
            Scrap::new("scrap1", &None, "# Scrap 1\n\nTagged #[[actual_tag]]."),
            Scrap::new(
                "scrap2",
                &None,
                "# Scrap 2\n\nThis links to [[actual_scrap]].",
            ),
            Scrap::new(
                "actual_scrap",
                &None,
                "# Actual Scrap\n\nThis is a regular scrap, not a tag.",
            ),
        ];

        let usecase = LookupTagBacklinksUsecase::new();

        // Request backlinks for "actual_scrap" — it's a scrap title, not a tag.
        // The scrap-link namespace and tag namespace are disjoint in v1, so
        // even though scrap2 wikilinks to it, this returns empty.
        let results = usecase
            .execute(&scraps, &Title::from("actual_scrap"))
            .expect("Should succeed");

        assert_eq!(
            results.len(),
            0,
            "Should return empty results for non-tag titles"
        );

        // Real tags work.
        let tag_results = usecase
            .execute(&scraps, &Title::from("actual_tag"))
            .expect("Should succeed");

        assert_eq!(
            tag_results.len(),
            1,
            "Should return results for actual tags"
        );
        assert_eq!(tag_results[0].title.to_string(), "scrap1");
    }

    #[test]
    fn test_lookup_tag_backlinks_empty_scraps_directory() {
        let scraps: Vec<Scrap> = vec![];

        let usecase = LookupTagBacklinksUsecase::new();

        let results = usecase
            .execute(&scraps, &Title::from("any_tag"))
            .expect("Should succeed");

        assert_eq!(results.len(), 0);
    }
}
