use std::collections::HashMap;

use crate::error::ScrapsResult;
use crate::usecase::scrap::scrap_ref::ScrapRef;
use scraps_libs::model::scrap::Scrap;

use scraps_libs::search::engine::{SearchEngine, SearchLogic};
use scraps_libs::search::fuzzy_engine::FuzzySearchEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub scrap: ScrapRef,
    pub description: Option<String>,
}

pub struct SearchUsecase;

impl SearchUsecase {
    pub fn new() -> SearchUsecase {
        SearchUsecase
    }

    pub fn execute(
        &self,
        scraps: &[Scrap],
        query: &str,
        num: usize,
        logic: SearchLogic,
    ) -> ScrapsResult<Vec<SearchResult>> {
        // Create key-to-scrap mapping for efficient lookup
        let scrap_map: HashMap<String, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key().to_string(), scrap))
            .collect();

        let lib_items: Vec<scraps_libs::search::result::SearchItem> = scraps
            .iter()
            .map(|scrap| {
                let tags: Vec<String> = scrap.tags().iter().map(ToString::to_string).collect();
                scraps_libs::search::result::SearchItem::new(
                    &scrap.self_key().to_string(),
                    scrap.frontmatter_title(),
                    scrap.description(),
                    &tags,
                )
            })
            .collect();

        // Perform search
        let engine = FuzzySearchEngine::new();
        let search_results = engine.search(&lib_items, query, num, logic);

        // Convert to final results using HashMap lookup
        let results: Vec<SearchResult> = search_results
            .into_iter()
            .filter_map(|result| {
                scrap_map.get(&result.key).map(|scrap| SearchResult {
                    scrap: (*scrap).into(),
                    description: scrap.description().map(str::to_string),
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
    fn it_run() {
        let scraps = vec![
            Scrap::new(
                "test1",
                &None,
                "# Test Document 1\nThis is a test document.",
            ),
            Scrap::new("test2", &None, "# Another Document\nAnother test content."),
        ];

        let usecase = SearchUsecase::new();

        let results = usecase
            .execute(&scraps, "test", 100, SearchLogic::And)
            .unwrap();

        // Should find documents containing "test"
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.scrap.title.contains("test")));
    }

    #[test]
    fn it_does_not_match_the_frontmatter_block() {
        let text = "---\nowner: zebra\n---\n\n# Orders\n";
        let scraps = vec![Scrap::new("orders", &None, text)];
        let usecase = SearchUsecase::new();

        let in_block = usecase
            .execute(&scraps, "zebra", 100, SearchLogic::And)
            .unwrap();
        assert!(in_block.is_empty());

        let by_key = usecase
            .execute(&scraps, "orders", 100, SearchLogic::And)
            .unwrap();
        assert_eq!(by_key.len(), 1);
    }

    #[test]
    fn it_handles_duplicate_titles() {
        // Two scraps with the same title but different contexts (ctx/ and root)
        let scraps = vec![
            Scrap::new(
                "duplicate",
                &Some("ctx".into()),
                "# Duplicate\nContent in ctx directory.",
            ),
            Scrap::new(
                "duplicate",
                &None,
                "# Duplicate\nContent in root directory.",
            ),
        ];

        let usecase = SearchUsecase::new();

        let results = usecase
            .execute(&scraps, "duplicate", 100, SearchLogic::And)
            .unwrap();

        // Should find both scraps with the same title but different contexts
        assert_eq!(
            results.len(),
            2,
            "Expected 2 results for duplicate titles with different contexts, got {}. \
            This indicates the HashMap is overwriting duplicate titles.",
            results.len()
        );

        // Verify ctx field separation works correctly
        // One result should have ctx: Some("ctx"), the other should have ctx: None
        let ctx_values: std::collections::HashSet<Option<String>> = results
            .iter()
            .map(|r| r.scrap.ctx().as_ref().map(|c| c.to_string()))
            .collect();
        assert_eq!(
            ctx_values.len(),
            2,
            "Expected 2 different ctx values (Some and None), got {:?}",
            ctx_values
        );

        // Verify that one result has ctx "ctx" and the other has None
        assert!(
            ctx_values.contains(&Some("ctx".to_string())),
            "Expected one result to have ctx 'ctx', but found {:?}",
            ctx_values
        );
        assert!(
            ctx_values.contains(&None),
            "Expected one result to have ctx None, but found {:?}",
            ctx_values
        );

        // Verify that both results have the same title "Duplicate"
        let titles: std::collections::HashSet<String> =
            results.iter().map(|r| r.scrap.title.clone()).collect();
        assert_eq!(
            titles.len(),
            1,
            "Expected all results to have the same title 'Duplicate', but got {:?}",
            titles
        );
        assert!(
            titles.contains("duplicate"),
            "Expected title to be 'duplicate', but got {:?}",
            titles
        );
    }

    #[test]
    fn it_leaves_the_body_unsearched() {
        let scraps = vec![Scrap::new(
            "mydoc",
            &None,
            "# mydoc\n\nThis body contains uniquekeyword here.",
        )];

        let results = SearchUsecase::new()
            .execute(&scraps, "uniquekeyword", 100, SearchLogic::And)
            .unwrap();

        assert!(results.is_empty(), "the body is read by get, not search");
    }

    #[test]
    fn it_matches_the_description() {
        let text = "---\ndescription: One row per completed customer order.\n---\n\n# Orders\n";
        let scraps = vec![Scrap::new("orders", &None, text)];

        let results = SearchUsecase::new()
            .execute(&scraps, "customer", 100, SearchLogic::And)
            .unwrap();

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn it_matches_a_tag() {
        let scraps = vec![
            Scrap::new("tokio", &None, "# tokio\n\n#[[Rust]] runtime"),
            Scrap::new("asyncio", &None, "# asyncio\n\n#[[Python]] runtime"),
        ];

        let results = SearchUsecase::new()
            .execute(&scraps, "rust", 100, SearchLogic::And)
            .unwrap();

        let keys: Vec<String> = results.iter().map(|r| r.scrap.title.clone()).collect();
        assert_eq!(keys, vec!["tokio".to_string()]);
    }
}
