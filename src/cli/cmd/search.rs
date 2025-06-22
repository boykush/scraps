use crate::{error::ScrapsResult, usecase::search::cmd::SearchCommand};
use url::Url;

pub fn run(query: &str) -> ScrapsResult<()> {
    // Use current directory as scraps directory
    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let scraps_dir_path = current_dir.join("scraps");
    let static_dir_path = current_dir.join("static");
    let public_dir_path = current_dir.join("public");

    // Use localhost URL for search (URLs are only used for URL generation in dynamic mode)
    let base_url = Url::parse("http://localhost:1112/").unwrap();

    let search_command = SearchCommand::new(&scraps_dir_path, &static_dir_path, &public_dir_path);
    let results = search_command.run(&base_url, query)?;

    if results.is_empty() {
        println!("No results found for query: {}", query);
    } else {
        println!("Found {} result(s) for query: {}", results.len(), query);
        println!();
        for result in results {
            println!("Title: {}", result.title);
            println!("URL: {}", result.url);
            println!();
        }
    }

    Ok(())
}
