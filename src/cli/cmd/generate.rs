use std::path::Path;

use scraps_libs::error::ScrapResult;

use crate::generate::cmd::GenerateCommand;

pub fn run(template_name: &str) -> ScrapResult<()> {
    let templates_dir_path = Path::new("templates");
    let scraps_dir_path = Path::new("scraps");

    let command = GenerateCommand::new(scraps_dir_path, templates_dir_path);

    command.run(template_name)?;

    Ok(())
}
