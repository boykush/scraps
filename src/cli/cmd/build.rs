use std::path::PathBuf;

use crate::build::cmd::{BuildCommand, HtmlMetadata};
use crate::libs::error::result::ScrapResult;

use crate::cli::scrap_config::ScrapConfig;

pub fn run() -> ScrapResult<()> {
    let config = ScrapConfig::new()?;

    let html_metadata = &HtmlMetadata {
        title: config.title,
        description: config.description,
    };

    let scraps_dir_path = PathBuf::from("scraps");
    let static_dir_path = PathBuf::from("static");
    let public_dir_path = PathBuf::from("public");

    BuildCommand::new(
        &html_metadata,
        &scraps_dir_path,
        &static_dir_path,
        &public_dir_path,
    )
    .run()
}
