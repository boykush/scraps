use clap_verbosity_flag::{log, Verbosity, WarnLevel};
use colored::Colorize;
use std::path::Path;
use std::time::Instant;
use tracing::{span, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use url::Url;

use crate::build::cmd::BuildCommand;
use crate::build::model::color_scheme::ColorScheme;
use crate::build::model::css::CssMetadata;
use crate::build::model::html::HtmlMetadata;
use crate::build::model::list_view_configs::ListViewConfigs;
use crate::build::model::paging::Paging;
use crate::build::model::sort::SortKey;
use crate::cli::config::color_scheme::ColorSchemeConfig;
use crate::cli::config::sort_key::SortKeyConfig;
use scraps_libs::error::ScrapResult;

use crate::cli::config::scrap_config::ScrapConfig;
use scraps_libs::git::GitCommandImpl;

pub fn run(verbose: Verbosity<WarnLevel>) -> ScrapResult<()> {
    let log_level = match verbose.log_level() {
        Some(log::Level::Error) => Level::ERROR,
        Some(log::Level::Warn) => Level::WARN,
        Some(log::Level::Info) => Level::INFO,
        Some(log::Level::Debug) => Level::DEBUG,
        Some(log::Level::Trace) => Level::TRACE,
        None => Level::WARN,
    };
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .with_max_level(log_level)
        .init();
    let span_run = span!(Level::INFO, "run").entered();

    let git_command = GitCommandImpl::new();
    let scraps_dir_path = Path::new("scraps");
    let static_dir_path = Path::new("static");
    let public_dir_path = Path::new("public");
    let command = BuildCommand::new(scraps_dir_path, static_dir_path, public_dir_path);

    let config = ScrapConfig::new()?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        Url::parse((config.base_url.to_string() + "/").as_str()).unwrap()
    };
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = HtmlMetadata::new(&config.title, &config.description, &config.favicon);
    let css_metadata = CssMetadata::new(&config.color_scheme.map_or_else(
        || ColorScheme::OsSetting,
        ColorSchemeConfig::into_color_scheme,
    ));
    let build_search_index = config.build_search_index.unwrap_or(true);
    let sort_key = config
        .sort_key
        .map_or_else(|| SortKey::CommittedDate, SortKeyConfig::into_sort_key);
    let paging = match config.paginate_by {
        None => Paging::Not,
        Some(u) => Paging::By(u),
    };
    let list_view_configs = ListViewConfigs::new(&build_search_index, &sort_key, &paging);

    let start = Instant::now();
    let result = command.run(
        git_command,
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    )?;
    let end = start.elapsed();

    span_run.exit();
    println!("-> Created {result} scraps");
    println!(
        "{} {}.{} {}",
        "Done in".green(),
        end.as_secs().to_string().green(),
        end.subsec_millis().to_string().green(),
        "secs".green()
    );
    Ok(())
}
