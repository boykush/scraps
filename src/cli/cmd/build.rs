use clap_verbosity_flag::{log, Verbosity, WarnLevel};
use std::path::Path;
use std::time::Instant;
use tracing::{span, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use url::Url;

use crate::cli::config::color_scheme::ColorSchemeConfig;
use crate::cli::config::sort_key::SortKeyConfig;
use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::usecase::build::model::color_scheme::ColorScheme;
use crate::usecase::build::model::css::CssMetadata;
use crate::usecase::build::model::html::HtmlMetadata;
use crate::usecase::build::model::list_view_configs::ListViewConfigs;
use crate::usecase::build::model::paging::Paging;
use crate::usecase::build::model::sort::SortKey;
use crate::usecase::build::usecase::BuildUsecase;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::usecase::progress::Progress;
use scraps_libs::git::GitCommandImpl;

pub fn run(verbose: Verbosity<WarnLevel>, project_path: Option<&Path>) -> ScrapsResult<()> {
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

    let path_resolver = PathResolver::new(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let static_dir_path = path_resolver.static_dir();
    let public_dir_path = path_resolver.public_dir();
    let usecase = BuildUsecase::new(&scraps_dir_path, &static_dir_path, &public_dir_path);

    let git_command = GitCommandImpl::new();
    let progress = ProgressImpl::init(Instant::now());

    let config = ScrapConfig::from_path(project_path)?;
    // Automatically append a trailing slash to URLs
    let base_url = if config.base_url.path().ends_with('/') {
        config.base_url
    } else {
        config.base_url.join("/").unwrap()
    };
    let lang_code = config
        .lang_code
        .map(|c| c.into_lang_code())
        .unwrap_or_default();
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = HtmlMetadata::new(
        &lang_code,
        &config.title,
        &config.description,
        &config.favicon,
    );
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

    usecase.execute(
        git_command,
        &progress,
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    )?;
    span_run.exit();
    progress.end();

    Ok(())
}
