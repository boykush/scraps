use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::ScrapsResult;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::file::ScrapFileStem;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::search::engine::SearchEngine;
use scraps_libs::search::fuzzy_engine::FuzzySearchEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
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
        base_url: &BaseUrl,
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
            .map(|scrap| (scrap.self_link().to_string(), scrap))
            .collect();

        // Create search items in memory
        let lib_items: Vec<scraps_libs::search::result::SearchItem> = scraps
            .iter()
            .map(|scrap| {
                scraps_libs::search::result::SearchItem::new(&scrap.self_link().to_string())
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
                    let file_stem = ScrapFileStem::from(scrap.self_link().clone());
                    let url = format!("{}scraps/{}.html", base_url.as_url(), file_stem);
                    SearchResult {
                        title: result.title,
                        url,
                        md_text: scrap.md_text.clone(),
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
    use url::Url;

    #[test]
    fn it_run() {
        // fields
        let test_resource_path = PathBuf::from("tests/resource/search/cmd/it_run");
        let scraps_dir_path = test_resource_path.join("scraps");
        let static_dir_path = test_resource_path.join("static");

        // scrap1
        let md_path_1 = scraps_dir_path.join("test1.md");
        let resource_bytes_1 =
            concat!("# Test Document 1\n", "This is a test document.",).as_bytes();

        // scrap2
        let md_path_2 = scraps_dir_path.join("test2.md");
        let resource_bytes_2 = concat!("# Another Document\n", "Another test content.").as_bytes();

        let mut test_resources = TestResources::new();
        test_resources
            .add_file(&md_path_1, resource_bytes_1)
            .add_file(&md_path_2, resource_bytes_2)
            .add_dir(&static_dir_path);

        test_resources.run(|| {
            let usecase = SearchUsecase::new(&scraps_dir_path);
            let url = Url::parse("http://localhost:1112/").unwrap();
            let base_url = BaseUrl::new(url).unwrap();

            let results = usecase.execute(&base_url, "test", 100).unwrap();

            // Should find documents containing "test"
            assert!(!results.is_empty());
            assert!(results.iter().any(|r| r.title.contains("test")));
            // Verify URLs are present
            assert!(results.iter().all(|r| !r.url.is_empty()));
            // Verify md_text is present
            assert!(results.iter().all(|r| !r.md_text.is_empty()));
        });
    }

    #[test]
    fn it_handles_duplicate_titles() {
        // fields
        let test_resource_path =
            PathBuf::from("tests/resource/search/cmd/it_handles_duplicate_titles");
        let scraps_dir_path = test_resource_path.join("scraps");
        let static_dir_path = test_resource_path.join("static");

        // Two scraps with the same title but different contexts (ctx/ and root)
        let ctx_dir = scraps_dir_path.join("ctx");
        let md_path_1 = ctx_dir.join("duplicate.md"); // ctx/duplicate.md
        let resource_bytes_1 = concat!("# Duplicate\n", "Content in ctx directory.").as_bytes();

        let md_path_2 = scraps_dir_path.join("duplicate.md"); // duplicate.md
        let resource_bytes_2 = concat!("# Duplicate\n", "Content in root directory.").as_bytes();

        let mut test_resources = TestResources::new();
        test_resources
            .add_dir(&ctx_dir)
            .add_file(&md_path_1, resource_bytes_1)
            .add_file(&md_path_2, resource_bytes_2)
            .add_dir(&static_dir_path);

        test_resources.run(|| {
            let usecase = SearchUsecase::new(&scraps_dir_path);
            let url = Url::parse("http://localhost:1112/").unwrap();
            let base_url = BaseUrl::new(url).unwrap();

            let results = usecase.execute(&base_url, "duplicate", 100).unwrap();

            // Should find both scraps with the same title but different contexts
            assert_eq!(
                results.len(),
                2,
                "Expected 2 results for duplicate titles with different contexts, got {}. \
                This indicates the HashMap is overwriting duplicate titles.",
                results.len()
            );

            // Verify URLs are different (pointing to different files)
            let urls: std::collections::HashSet<String> =
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
        });
    }
}
