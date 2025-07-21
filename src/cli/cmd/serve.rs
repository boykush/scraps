use std::time::Instant;
use std::{net::SocketAddr, path::Path};

use url::Url;

use crate::cli::path_resolver::PathResolver;
use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::usecase::progress::Progress;
use crate::{
    cli::config::{
        color_scheme::ColorSchemeConfig, scrap_config::ScrapConfig, sort_key::SortKeyConfig,
    },
    usecase::build::{
        model::{
            color_scheme::ColorScheme, css::CssMetadata, html::HtmlMetadata, list_view_configs,
            paging::Paging, sort::SortKey,
        },
        usecase::BuildUsecase,
    },
    usecase::serve::usecase::ServeUsecase,
};
use scraps_libs::git::GitCommandImpl;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    // set local environment
    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    let base_url = Url::parse(&format!("http://{addr}"))?.join("").unwrap();

    // resolve paths
    let path_resolver = PathResolver::new(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let static_dir_path = path_resolver.static_dir();
    let public_dir_path = path_resolver.public_dir();
    let build_usecase = BuildUsecase::new(&scraps_dir_path, &static_dir_path, &public_dir_path);

    let git_command = GitCommandImpl::new();
    let progress = ProgressImpl::init(Instant::now());

    let config = ScrapConfig::from_path(project_path)?;
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

    let build_result = build_usecase.execute(
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
    let serve_usecase = ServeUsecase::new(&public_dir_path);
    let serve_result = serve_usecase.execute(&addr);

    // merge result
    build_result.and(serve_result)
}
