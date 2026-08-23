use std::time::Instant;
use std::{net::SocketAddr, path::Path};

use scraps_libs::model::base_url::BaseUrl;
use url::Url;

use crate::cli::display::serve::DisplayServeInfo;
use crate::cli::path_resolver::PathResolver;
use crate::cli::progress::ProgressImpl;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::output::build_renderer::BuildRendererImpl;
use crate::usecase::progress::Progress;
use crate::{
    cli::config::scrap_config::ScrapConfig, usecase::build::usecase::BuildUsecase,
    usecase::serve::usecase::ServeUsecase,
};
use scraps_libs::git::GitCommandImpl;

pub fn run(git: bool, project_path: Option<&Path>) -> ScrapsResult<()> {
    // set local environment
    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    let base_url = BaseUrl::new(Url::parse(&format!("http://{addr}"))?.join("").unwrap()).unwrap();

    // resolve paths
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    // serve renders against the local address, not the configured base_url.
    let build_config = config.to_build_config(Some(base_url))?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let static_dir_path = path_resolver.static_dir();
    let output_dir_path = path_resolver.output_dir(&config);

    // Input: read scraps (with git timestamps if --git is set) and README.
    // The wiki root is the project root, so skip `static/` and the configured
    // output directory at the top level.
    let git_command = git.then(GitCommandImpl::new);
    let exclude_dirs = vec![static_dir_path.clone(), output_dir_path.clone()];
    let (scraps_with_ts, readme_text) =
        read_scraps::to_all_scraps_with_timestamps(&scraps_dir_path, &exclude_dirs, git_command)?;

    let renderer = BuildRendererImpl::new(&static_dir_path, &output_dir_path)?;
    let build_usecase = BuildUsecase::new();

    let progress = ProgressImpl::init(Instant::now());

    let scrap_count = build_usecase.execute(
        &scraps_with_ts,
        &readme_text,
        &progress,
        &renderer,
        &build_config,
    )?;
    progress.end();

    // display serve info
    let title = build_config.html_metadata.title();
    let serve_info = DisplayServeInfo::new(&title, &format!("http://{addr}"), scrap_count);
    println!("{serve_info}");

    // serve command
    let serve_usecase = ServeUsecase::new(&output_dir_path);
    serve_usecase.execute(&addr)
}
