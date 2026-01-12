use std::path::Path;

use itertools::Itertools;

use crate::cli::display::tag::DisplayTag;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::usecase::tag::list::usecase::ListTagUsecase;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let usecase = ListTagUsecase::new(&scraps_dir_path);

    let base_url = config.base_url.into_base_url();

    let (tags, backlinks_map) = usecase.execute()?;
    let display_tags_result = tags
        .into_iter()
        .map(|tag| DisplayTag::new(&tag, Some(&base_url), &backlinks_map))
        .collect::<ScrapsResult<Vec<DisplayTag>>>();

    display_tags_result.map(|tags| {
        let sorted = tags
            .into_iter()
            .sorted_by_key(|tag| tag.backlinks_count())
            .rev();
        for tag in sorted {
            println!("{tag}")
        }
    })
}
