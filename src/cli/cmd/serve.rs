use std::time::Instant;
use std::{net::SocketAddr, path::Path};

use url::Url;

use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::usecase::progress::Progress;
use crate::{
    cli::config::{
        color_scheme::ColorSchemeConfig, scrap_config::ScrapConfig, sort_key::SortKeyConfig,
    },
    usecase::build::{
        cmd::BuildCommand,
        model::{
            color_scheme::ColorScheme, css::CssMetadata, html::HtmlMetadata, list_view_configs,
            paging::Paging, sort::SortKey,
        },
    },
    usecase::serve::cmd::ServeCommand,
};
use scraps_libs::git::GitCommandImpl;

pub fn run() -> ScrapsResult<()> {
    // set local environment
    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    let base_url = Url::parse(&format!("http://{}", addr))?.join("").unwrap();

    // build command
    let scraps_dir_path = Path::new("scraps");
    let static_dir_path = Path::new("static");
    let public_dir_path = Path::new("public");
    let build_command = BuildCommand::new(scraps_dir_path, static_dir_path, public_dir_path);

    let git_command = GitCommandImpl::new();
    let progress = ProgressImpl::init(Instant::now());

    let config = ScrapConfig::new()?;
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
    let list_view_configs =
        list_view_configs::ListViewConfigs::new(&build_search_index, &sort_key, &paging);

    let build_result = build_command.run(
        git_command,
        &progress,
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    );
    progress.end();

    // serve command
    let serve_command = ServeCommand::new(public_dir_path);
    let serve_result = serve_command.run(&addr);

    // merge result
    build_result.and(serve_result)
}
