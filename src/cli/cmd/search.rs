use std::path::Path;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::display::search::DisplaySearch;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::search::usecase::SearchUsecase;

pub fn run(query: &str, num: usize, project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);

    let search_usecase = SearchUsecase::new(&scraps_dir_path);
    let results = search_usecase.execute(query, num)?;

    if results.is_empty() {
        println!("No results found for query: {query}");
    } else {
        for result in results {
            // Construct file path from title and ctx
            let file_path = if let Some(ctx) = &result.ctx {
                scraps_dir_path
                    .join(ctx.to_string())
                    .join(format!("{}.md", result.title))
            } else {
                scraps_dir_path.join(format!("{}.md", result.title))
            };

            let display_search = DisplaySearch::new_with_file_path(&result, &file_path);
            println!("{display_search}");
        }
    }

    Ok(())
}
