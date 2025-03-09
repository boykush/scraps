use std::path::PathBuf;

use itertools::Itertools;
use url::Url;

use crate::cli::display::tag::DisplayTag;
use crate::error::ScrapsResult;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::tag::cmd::TagCommand;

pub fn run() -> ScrapsResult<()> {
    let scraps_dir_path = PathBuf::from("scraps");
    let command = TagCommand::new(&scraps_dir_path);

    let config = ScrapConfig::new()?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        Url::parse((config.base_url.to_string() + "/").as_str()).unwrap()
    };

    let (tags, linked_scraps_map) = command.run(&base_url)?;
    let display_tags_result = tags
        .into_iter()
        .map(|tag| DisplayTag::new(&tag, &base_url, &linked_scraps_map))
        .collect::<ScrapsResult<Vec<DisplayTag>>>();

    display_tags_result.map(|tags| {
        let sorted = tags
            .into_iter()
            .sorted_by_key(|tag| tag.linked_count())
            .rev();
        for tag in sorted {
            println!("{}", tag)
        }
    })
}
