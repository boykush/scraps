use std::path::PathBuf;

use url::Url;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::error::ScrapsResult;
use crate::usecase::search::cmd::SearchCommand;

pub fn run(query: &str) -> ScrapsResult<()> {
    let scraps_dir_path = PathBuf::from("scraps");

    let config = ScrapConfig::new()?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        Url::parse((config.base_url.to_string() + "/").as_str()).unwrap()
    };

    let search_command = SearchCommand::new(&scraps_dir_path);
    let results = search_command.run(&base_url, query)?;

    if results.is_empty() {
        println!("No results found for query: {}", query);
    } else {
        for result in results {
            println!("{} {}", result.title, result.url);
        }
    }

    Ok(())
}
