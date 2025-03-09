use std::path::Path;

use crate::error::ScrapsResult;
use scraps_libs::model::title::Title;

use crate::{cli::config::scrap_config::ScrapConfig, template::generate::cmd::GenerateCommand};

pub fn run(template_name: &str, scrap_title: &Option<Title>) -> ScrapsResult<()> {
    let templates_dir_path = Path::new("templates");
    let scraps_dir_path = Path::new("scraps");

    let command = GenerateCommand::new(scraps_dir_path, templates_dir_path);

    let config = ScrapConfig::new()?;
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    command.run(template_name, scrap_title, &timezone)?;

    Ok(())
}
