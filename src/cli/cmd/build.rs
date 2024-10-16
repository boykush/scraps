use colored::Colorize;
use url::Url;
use std::path::PathBuf;
use std::time::Instant;

use crate::build::cmd::{BuildCommand, HtmlMetadata};
use crate::build::model::paging::Paging;
use crate::build::model::sort::SortKey;
use crate::libs::error::ScrapResult;

use crate::cli::scrap_config::{ScrapConfig, SortKeyConfig};
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
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with("/") {
        config.base_url
    } else { Url::parse((config.base_url.to_string() + "/").as_str()).unwrap() };
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = HtmlMetadata::new(&config.title, &config.description, &config.favicon);
    let sort_key = config
        .sort_key
        .map_or_else(|| SortKey::CommittedDate, SortKeyConfig::into_sort_key);
    let paging = match config.paginate_by {
        None => Paging::Not,
        Some(u) => Paging::By(u),
    };

    println!("{}", "Building site...".bold());
    let start = Instant::now();

    let result = command.run(&base_url, timezone, &html_metadata, &sort_key, &paging)?;

    let end = start.elapsed();
    println!("-> Created {result} scraps");
    Ok(println!(
        "{} {}.{} {}",
        "Done in".green(),
        end.as_secs().to_string().green(),
        end.subsec_millis().to_string().green(),
        "secs".green(),
    ))
}
