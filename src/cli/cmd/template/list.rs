use std::path::Path;

use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;

use crate::usecase::template::list::cmd::ListCommand;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let templates_dir_path = path_resolver.templates_dir();

    let command = ListCommand::new(&templates_dir_path);
    let template_names = command.run()?;

    for template_name in template_names {
        println!("{template_name}");
    }

    Ok(())
}
