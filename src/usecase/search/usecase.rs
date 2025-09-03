use std::path::PathBuf;

use crate::error::ScrapsResult;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::file::ScrapFileStem;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::search::engine::SearchEngine;
use scraps_libs::search::fuzzy_engine::FuzzySearchEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultWithUrl {
    pub title: String,
    pub url: String,
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
    ) -> ScrapsResult<Vec<SearchResultWithUrl>> {
        // Load scraps from directory directly
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Create search index items in memory
        let lib_items: Vec<scraps_libs::search::result::SearchIndexItem> = scraps
            .iter()
            .map(|scrap| {
                scraps_libs::search::result::SearchIndexItem::new(&scrap.title.to_string())
            })
            .collect();

        // Perform search and add URLs to results
        let engine = FuzzySearchEngine::new();
        let search_results = engine.search(&lib_items, query, num);

        // Convert to final results with URLs
        let results_with_urls: Vec<SearchResultWithUrl> = search_results
            .into_iter()
            .enumerate()
            .map(|(index, _result)| {
                let scrap = &scraps[index];
                let file_stem = ScrapFileStem::from(scrap.self_link().clone());
                let url = format!("{}scraps/{}.html", base_url.as_url(), file_stem);
                SearchResultWithUrl {
                    title: scrap.title.to_string(),
                    url,
                }
            })
            .collect();

        Ok(results_with_urls)
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
        });
    }
}
