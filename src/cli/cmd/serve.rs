use std::{net::SocketAddr, path::PathBuf};

use url::Url;

use crate::{
    build::{
        cmd::{BuildCommand, HtmlMetadata},
        model::{paging::Paging, sort::SortKey},
    },
    cli::scrap_config::{ScrapConfig, SortKeyConfig},
    libs::{error::ScrapResult, git::GitCommandImpl},
    serve::cmd::ServeCommand,
};

pub fn run() -> ScrapResult<()> {
    // set local environment
    let addr: SocketAddr = ([127, 0, 0, 1], 1112).into();
    let base_url = Url::parse(&format!("http://{}", addr.to_string()))?.join("").unwrap();

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
    let sort_key = config
        .sort_key
        .map_or_else(|| SortKey::CommittedDate, SortKeyConfig::into_sort_key);
    let paging = match config.paginate_by {
        None => Paging::Not,
        Some(u) => Paging::By(u),
    };
    let build_result = build_command.run(&base_url, timezone, &html_metadata, &sort_key, &paging);

    // serve command
    let serve_command = ServeCommand::new(&public_dir_path);
    let serve_result = serve_command.run(&addr);

    // merge result
    build_result.and(serve_result)
}
