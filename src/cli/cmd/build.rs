use std::path::PathBuf;

use crate::build::cmd::{BuildCommand, HtmlMetadata};
use crate::build::model::sort::SortKey;
use crate::libs::error::result::ScrapResult;

use crate::cli::scrap_config::ScrapConfig;
use crate::libs::git::GitCommandImpl;

pub fn run() -> ScrapResult<()> {
    let git_command = GitCommandImpl::new();
    let scraps_dir_path = PathBuf::from("scraps");
    let static_dir_path = PathBuf::from("static");
    let public_dir_path = PathBuf::from("public");
    let command = BuildCommand::new(
        git_command,
        &scraps_dir_path,
        &static_dir_path,
        &public_dir_path,
    );

    let config = ScrapConfig::new()?;
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = &HtmlMetadata {
        title: config.title,
        description: config.description,
        favicon: config.favicon,
    };
    let sort_key = config
        .sort_key
        .map_or_else(|| SortKey::CommitedDate, |c| c.into_sort_key());
    command.run(&timezone, &html_metadata, &sort_key)
}
