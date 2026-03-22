use clap_verbosity_flag::{log, Verbosity, WarnLevel};
use std::path::Path;
use std::time::Instant;
use tracing::{span, Level};
use tracing_subscriber::fmt::format::FmtSpan;

use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::output::file::{
    index_page_writer::FileIndexPageWriter, scrap_page_writer::FileScrapPageWriter,
    search_index_writer::FileSearchIndexWriter, style_writer::FileStyleWriter,
    tag_page_writer::FileTagPageWriter,
};
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
    let config = ScrapConfig::from_path(project_path)?;
    let ssg = config.require_ssg()?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let static_dir_path = path_resolver.static_dir();
    let public_dir_path = path_resolver.public_dir();

    // Input: read scraps with git timestamps and README
    let git_command = GitCommandImpl::new();
    let (scraps_with_ts, readme_text) =
        read_scraps::to_all_scraps_with_timestamps(&scraps_dir_path, git_command)?;

    let base_url = ssg.base_url();
    let title = &ssg.title;
    let default_lang_code = scraps_libs::lang::LangCode::default();
    let lang_code = ssg
        .lang_code
        .as_ref()
        .map(|c| c.as_lang_code())
        .unwrap_or(&default_lang_code);
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    let html_metadata = HtmlMetadata::new(lang_code, title, &ssg.description, &ssg.favicon);
    let default_color_scheme = ColorScheme::OsSetting;
    let css_metadata = CssMetadata::new(
        ssg.color_scheme
            .as_ref()
            .map(|c| c.as_color_scheme())
            .unwrap_or(&default_color_scheme),
    );
    let build_search_index = ssg.build_search_index.unwrap_or(true);
    let default_sort_key = SortKey::CommittedDate;
    let sort_key = ssg
        .sort_key
        .as_ref()
        .map(|s| s.as_sort_key())
        .unwrap_or(&default_sort_key);
    let paging = match ssg.paginate_by {
        None => Paging::Not,
        Some(u) => Paging::By(u),
    };
    let list_view_configs = ListViewConfigs::new(&build_search_index, sort_key, &paging);

    let index_page_writer = FileIndexPageWriter::new(
        &static_dir_path,
        &public_dir_path,
        base_url.clone(),
        html_metadata.clone(),
    );
    let scrap_page_writer = FileScrapPageWriter::new(
        &static_dir_path,
        &public_dir_path,
        base_url.clone(),
        timezone,
        html_metadata.clone(),
    );
    let tag_page_writer = FileTagPageWriter::new(
        &static_dir_path,
        &public_dir_path,
        base_url.clone(),
        html_metadata,
    );
    let style_writer = FileStyleWriter::new(&static_dir_path, &public_dir_path, css_metadata);
    let search_index_writer =
        FileSearchIndexWriter::new(&static_dir_path, &public_dir_path, base_url.clone());
    let progress = ProgressImpl::init(Instant::now());
    let usecase = BuildUsecase::new(
        &index_page_writer,
        &scrap_page_writer,
        &tag_page_writer,
        &style_writer,
        &search_index_writer,
        &progress,
    );
    usecase.execute(&scraps_with_ts, &readme_text, &base_url, &list_view_configs)?;
    span_run.exit();
    progress.end();

    Ok(())
}
