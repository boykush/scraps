use std::path::Path;

use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use scraps_libs::model::title::Title;

use crate::{
    cli::config::scrap_config::ScrapConfig, usecase::template::generate::cmd::GenerateCommand,
};

pub fn run(
    template_name: &str,
    scrap_title: &Option<Title>,
    project_path: Option<&Path>,
) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let templates_dir_path = path_resolver.templates_dir();
    let scraps_dir_path = path_resolver.scraps_dir();

    let command = GenerateCommand::new(&scraps_dir_path, &templates_dir_path);

    let config = ScrapConfig::from_path(project_path)?;
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    command.run(template_name, scrap_title, &timezone)?;

    Ok(())
}
