use std::path::Path;

use crate::error::ScrapsResult;

use crate::template::list::cmd::ListCommand;

pub fn run() -> ScrapsResult<()> {
    let templates_dir_path = Path::new("templates");

    let command = ListCommand::new(templates_dir_path);
    let template_names = command.run()?;

    for template_name in template_names {
        println!("{}", template_name);
    }

    Ok(())
}
