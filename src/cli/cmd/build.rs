use std::path::PathBuf;

use crate::build::cmd::BuildCommand;
use crate::libs::error::result::ScrapResult;

use crate::cli::scrap_config::ScrapConfig;

pub fn run() -> ScrapResult<()> {
    let config = ScrapConfig::new()?;

    let site_title = config.title;
    let scraps_dir_path = PathBuf::from("scraps");
    let static_dir_path = PathBuf::from("static");
    let public_dir_path = PathBuf::from("public");
    BuildCommand::new(
        &site_title,
        &scraps_dir_path,
        &static_dir_path,
        &public_dir_path,
    )
    .run()
}
