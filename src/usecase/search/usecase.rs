use std::path::PathBuf;

use crate::error::ScrapsResult;
use crate::service::search::render::SearchIndexRender;
use scraps_libs::model::base_url::BaseUrl;
use scraps_libs::model::scrap::Scrap;
use scraps_libs::search::engine::SearchEngine;
use scraps_libs::search::fuzzy_engine::FuzzySearchEngine;
use scraps_libs::search::result::SearchResult;
use url::Url;

pub struct SearchUsecase {
    scraps_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl SearchUsecase {
    pub fn new(scraps_dir_path: &PathBuf, public_dir_path: &PathBuf) -> SearchUsecase {
        SearchUsecase {
            scraps_dir_path: scraps_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    pub fn execute(
        &self,
        base_url: &BaseUrl,
        query: &str,
        num: usize,
    ) -> ScrapsResult<Vec<SearchResult>> {
        Self::build_search_index(self, base_url.as_url())?;
        let results = Self::perform_search(self, query, num);
        Ok(results)
    }

    fn build_search_index(&self, base_url: &Url) -> ScrapsResult<()> {
        // Load scraps from directory
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Render search index JSON
        SearchIndexRender::new(&self.scraps_dir_path, &self.public_dir_path)?.run(base_url, &scraps)
    }

    fn perform_search(&self, query: &str, num: usize) -> Vec<SearchResult> {
        let search_index_path = self.public_dir_path.join("search_index.json");

        let indexed_str = std::fs::read_to_string(&search_index_path).unwrap();
        let items: Vec<SearchIndexItem> = serde_json::from_str(&indexed_str).unwrap();

        // Convert to lib types
        let lib_items: Vec<scraps_libs::search::result::SearchIndexItem> =
            items.into_iter().map(|item| item.into_lib_type()).collect();

        let engine = FuzzySearchEngine::new();
        engine.search(&lib_items, query, num)
    }
}

#[derive(serde::Deserialize)]
#[serde(remote = "scraps_libs::search::result::SearchIndexItem")]
struct SerdeSearchIndexItem {
    title: String,
    url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SearchIndexItem(
    #[serde(with = "SerdeSearchIndexItem")] scraps_libs::search::result::SearchIndexItem,
);

impl SearchIndexItem {
    fn into_lib_type(self) -> scraps_libs::search::result::SearchIndexItem {
        self.0
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
        let public_dir_path = test_resource_path.join("public");

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
            .add_dir(&static_dir_path)
            .add_dir(&public_dir_path);

        test_resources.run(|| {
            let usecase = SearchUsecase::new(&scraps_dir_path, &public_dir_path);
            let url = Url::parse("http://localhost:1112/").unwrap();
            let base_url = BaseUrl::new(url).unwrap();

            let results = usecase.execute(&base_url, "test", 100).unwrap();

            // Should find documents containing "test"
            assert!(!results.is_empty());
            assert!(results.iter().any(|r| r.title.contains("test")));
        });
    }
}
