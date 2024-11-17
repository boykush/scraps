use std::path::PathBuf;
use url::Url;

use crate::libs::error::ScrapResult;

use crate::cli::scrap_config::ScrapConfig;
use crate::tag::cmd::TagCommand;

pub fn run() -> ScrapResult<()> {
    let scraps_dir_path = PathBuf::from("scraps");
    let command = TagCommand::new(
        &scraps_dir_path,
    );

    let config = ScrapConfig::new()?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        Url::parse((config.base_url.to_string() + "/").as_str()).unwrap()
    };

    let result = command.run(&base_url)?;

    Ok(println!(
        "{:?}",
        result,
    ))
}
