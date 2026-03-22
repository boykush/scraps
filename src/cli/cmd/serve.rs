use std::time::Instant;
use std::{net::SocketAddr, path::Path};

use scraps_libs::model::base_url::BaseUrl;
use url::Url;

use crate::cli::display::serve::DisplayServeInfo;
use crate::cli::path_resolver::PathResolver;
use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::progress::Progress;
use crate::{
    cli::config::scrap_config::ScrapConfig,
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
    let base_url = BaseUrl::new(Url::parse(&format!("http://{addr}"))?.join("").unwrap()).unwrap();

    // resolve paths
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

    let build_usecase = BuildUsecase::new(&static_dir_path, &public_dir_path);

    let progress = ProgressImpl::init(Instant::now());
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
    let list_view_configs =
        list_view_configs::ListViewConfigs::new(&build_search_index, sort_key, &paging);

    let scrap_count = build_usecase.execute(
        &scraps_with_ts,
        &readme_text,
        &progress,
        &base_url,
        timezone,
        &html_metadata,
        &css_metadata,
        &list_view_configs,
    )?;
    progress.end();

    // display serve info
    let serve_info = DisplayServeInfo::new(title, &format!("http://{addr}"), scrap_count);
    println!("{serve_info}");

    // serve command
    let serve_usecase = ServeUsecase::new(&public_dir_path);
    serve_usecase.execute(&addr)
}
