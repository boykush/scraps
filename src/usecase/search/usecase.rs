use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::ScrapsResult;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::file::ScrapFileStem;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::model::title::Title;
use scraps_libs::search::engine::SearchEngine;
use scraps_libs::search::fuzzy_engine::FuzzySearchEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub title: Title,
    pub ctx: Option<Ctx>,
    pub url: Option<String>,
    pub md_text: String,
}

pub struct SearchUsecase {
    scraps_dir_path: PathBuf,
}

impl SearchUsecase {
    pub fn new(scraps_dir_path: &PathBuf) -> SearchUsecase {
        SearchUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
        }
    }

    pub fn execute(
        &self,
        base_url: Option<&BaseUrl>,
        query: &str,
        num: usize,
    ) -> ScrapsResult<Vec<SearchResult>> {
        // Load scraps from directory directly
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Create title-to-scrap mapping for efficient lookup
        let scrap_map: HashMap<String, &Scrap> = scraps
            .iter()
            .map(|scrap| (scrap.self_key().to_string(), scrap))
            .collect();

        // Create search items in memory
        let lib_items: Vec<scraps_libs::search::result::SearchItem> = scraps
            .iter()
            .map(|scrap| {
                scraps_libs::search::result::SearchItem::new(&scrap.self_key().to_string())
            })
            .collect();

        // Perform search and add URLs to results
        let engine = FuzzySearchEngine::new();
        let search_results = engine.search(&lib_items, query, num);

        // Convert to final results with URLs using HashMap lookup
        let results: Vec<SearchResult> = search_results
            .into_iter()
            .filter_map(|result| {
                // Find the corresponding scrap by title using HashMap
                scrap_map.get(&result.title).map(|scrap| {
                    let url = base_url.map(|base_url| {
                        let file_stem = ScrapFileStem::from(scrap.self_key().clone());
                        format!("{}scraps/{}.html", base_url.as_url(), file_stem)
                    });

                    let scrap_key = &scrap.self_key();
                    let title: Title = scrap_key.into();
                    let ctx: Option<Ctx> = scrap_key.into();

                    SearchResult {
                        title,
                        ctx,
                        url,
                        md_text: scrap.md_text().to_string(),
                    }
                })
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_fixtures::TempScrapProject;

    use super::*;
    use url::Url;

    #[test]
    fn it_run() {
        let project = TempScrapProject::new();

        project
            .add_scrap("test1.md", b"# Test Document 1\nThis is a test document.")
            .add_scrap("test2.md", b"# Another Document\nAnother test content.");

        let usecase = SearchUsecase::new(&project.scraps_dir);
        let url = Url::parse("http://localhost:1112/").unwrap();
        let base_url = BaseUrl::new(url).unwrap();

        let results = usecase.execute(Some(&base_url), "test", 100).unwrap();

        // Should find documents containing "test"
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.title.to_string().contains("test")));
        // Verify URLs are present when base_url is provided
        assert!(results
            .iter()
            .all(|r| r.url.is_some() && !r.url.as_ref().unwrap().is_empty()));
        // Verify md_text is present
        assert!(results.iter().all(|r| !r.md_text.is_empty()));
    }

    #[test]
    fn it_handles_duplicate_titles() {
        let project = TempScrapProject::new();

        // Two scraps with the same title but different contexts (ctx/ and root)
        project
            .add_scrap_with_context(
                "ctx",
                "duplicate.md",
                b"# Duplicate\nContent in ctx directory.",
            )
            .add_scrap("duplicate.md", b"# Duplicate\nContent in root directory.");

        let usecase = SearchUsecase::new(&project.scraps_dir);
        let url = Url::parse("http://localhost:1112/").unwrap();
        let base_url = BaseUrl::new(url).unwrap();

        let results = usecase.execute(Some(&base_url), "duplicate", 100).unwrap();

        // Should find both scraps with the same title but different contexts
        assert_eq!(
            results.len(),
            2,
            "Expected 2 results for duplicate titles with different contexts, got {}. \
            This indicates the HashMap is overwriting duplicate titles.",
            results.len()
        );

        // Verify URLs are different (pointing to different files)
        let urls: std::collections::HashSet<Option<String>> =
            results.iter().map(|r| r.url.clone()).collect();
        assert_eq!(urls.len(), 2, "Expected 2 unique URLs, got {}", urls.len());

        // Verify content is different
        let md_texts: std::collections::HashSet<String> =
            results.iter().map(|r| r.md_text.clone()).collect();
        assert_eq!(
            md_texts.len(),
            2,
            "Expected 2 unique content texts, got {}",
            md_texts.len()
        );

        // Verify ctx field separation works correctly
        // One result should have ctx: Some("ctx"), the other should have ctx: None
        let ctx_values: std::collections::HashSet<Option<String>> = results
            .iter()
            .map(|r| r.ctx.as_ref().map(|c| c.to_string()))
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
            results.iter().map(|r| r.title.to_string()).collect();
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
}
