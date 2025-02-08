use std::{net::SocketAddr, path::PathBuf};

use url::Url;

use crate::{
    build::{
        cmd::BuildCommand,
        model::{
            color_scheme::ColorScheme, css::CssMetadata, html::HtmlMetadata, list_view_configs,
            paging::Paging, sort::SortKey,
        },
    },
    cli::config::{
        color_scheme::ColorSchemeConfig, scrap_config::ScrapConfig, sort_key::SortKeyConfig,
    },
    serve::cmd::ServeCommand,
};
use scraps_libs::{error::ScrapResult, git::GitCommandImpl};

pub fn run() -> ScrapResult<()> {
    // set local environment
    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    let base_url = Url::parse(&format!("http://{}", addr))?.join("").unwrap();

    // build command
    let git_command = GitCommandImpl::new();
    let scraps_dir_path = PathBuf::from("scraps");
    let static_dir_path = PathBuf::from("static");
    let public_dir_path = PathBuf::from("public");
    let build_command = BuildCommand::new(
        git_command,
        &scraps_dir_path,
        &static_dir_path,
        &public_dir_path,
    );
    let config = ScrapConfig::new()?;
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
    let list_view_configs =
        list_view_configs::ListViewConfigs::new(&build_search_index, &sort_key, &paging);

    let build_result = build_command.run(
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    );

    // serve command
    let serve_command = ServeCommand::new(&public_dir_path);
    let serve_result = serve_command.run(&addr);

    // merge result
    build_result.and(serve_result)
}
