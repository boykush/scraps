use std::{fs, path::PathBuf};

use crate::error::{anyhow::Context, BuildError, ScrapsResult};
use crate::service::search::{
    render::SearchIndexRender, serde::search_index_scraps::SearchIndexScrapsTera,
};
use scraps_libs::model::scrap::Scrap;
use url::Url;

pub struct SearchCommand {
    scraps_dir_path: PathBuf,
    static_dir_path: PathBuf,
    public_dir_path: PathBuf,
}

impl SearchCommand {
    pub fn new(
        scraps_dir_path: &PathBuf,
        static_dir_path: &PathBuf,
        public_dir_path: &PathBuf,
    ) -> SearchCommand {
        SearchCommand {
            scraps_dir_path: scraps_dir_path.to_owned(),
            static_dir_path: static_dir_path.to_owned(),
            public_dir_path: public_dir_path.to_owned(),
        }
    }

    pub fn run(&self, base_url: &Url, query: &str) -> ScrapsResult<Vec<SearchResult>> {
        let search_data = self.get_or_generate_search_data(base_url)?;
        let results = Self::perform_search(&search_data, query);
        Ok(results)
    }

    fn get_or_generate_search_data(&self, base_url: &Url) -> ScrapsResult<Vec<SearchIndexItem>> {
        let search_index_path = self.public_dir_path.join("search_index.json");

        if search_index_path.exists() {
            // Load from existing JSON file
            Self::load_search_data_from_json(&search_index_path)
        } else {
            // Generate search data dynamically
            self.generate_search_data(base_url)
        }
    }

    fn load_search_data_from_json(json_path: &PathBuf) -> ScrapsResult<Vec<SearchIndexItem>> {
        let json_content = fs::read_to_string(json_path).context(BuildError::ReadREADMEFile)?;
        let items: Vec<SearchIndexItem> =
            serde_json::from_str(&json_content).context(BuildError::ReadREADMEFile)?;
        Ok(items)
    }

    fn generate_search_data(&self, base_url: &Url) -> ScrapsResult<Vec<SearchIndexItem>> {
        // Load scraps from directory
        let scrap_paths = crate::usecase::read_scraps::to_scrap_paths(&self.scraps_dir_path)?;
        let scraps = scrap_paths
            .into_iter()
            .map(|path| crate::usecase::read_scraps::to_scrap_by_path(&self.scraps_dir_path, &path))
            .collect::<ScrapsResult<Vec<Scrap>>>()?;

        // Generate search data using SearchIndexRender
        let search_data = SearchIndexRender::generate_search_data(&scraps);

        // Convert to internal format
        let items = Self::convert_to_search_items(&search_data, base_url);
        Ok(items)
    }

    fn convert_to_search_items(
        data: &SearchIndexScrapsTera,
        base_url: &Url,
    ) -> Vec<SearchIndexItem> {
        // Convert SearchIndexScrapsTera to SearchIndexItem via JSON serialization
        // since direct field access is not available with current implementation
        let json_str = serde_json::to_string(data).unwrap();
        let scrap_items: Vec<SearchIndexScrapItem> = serde_json::from_str(&json_str).unwrap();

        scrap_items
            .into_iter()
            .map(|item| SearchIndexItem {
                title: item.link_title,
                url: format!("{}scraps/{}.html", base_url, item.file_stem),
            })
            .collect()
    }

    fn perform_search(items: &[SearchIndexItem], query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();

        items
            .iter()
            .filter(|item| item.title.to_lowercase().contains(&query_lower))
            .map(|item| SearchResult {
                title: item.title.clone(),
                url: item.url.clone(),
            })
            .collect()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SearchIndexItem {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SearchIndexScrapItem {
    link_title: String,
    file_stem: String,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
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
            let command = SearchCommand::new(&scraps_dir_path, &static_dir_path, &public_dir_path);
            let base_url = Url::parse("http://localhost:1112/").unwrap();

            let results = command.run(&base_url, "test").unwrap();

            // Should find documents containing "test"
            assert!(!results.is_empty());
            assert!(results.iter().any(|r| r.title.contains("test")));
        });
    }
}
