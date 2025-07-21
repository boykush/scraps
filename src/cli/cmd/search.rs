use std::path::Path;

use url::Url;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::display::search::DisplaySearch;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::search::usecase::SearchUsecase;

pub fn run(query: &str, num: usize, project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let public_dir_path = path_resolver.public_dir();

    let config = ScrapConfig::from_path(project_path)?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        Url::parse((config.base_url.to_string() + "/").as_str()).unwrap()
    };

    let search_usecase = SearchUsecase::new(&scraps_dir_path, &public_dir_path);
    let results = search_usecase.execute(&base_url, query, num)?;

    if results.is_empty() {
        println!("No results found for query: {query}");
    } else {
        for result in results {
            let display_search = DisplaySearch::new(&result);
            println!("{display_search}");
        }
    }

    Ok(())
}
