use crate::{init::cmd::InitCommand, libs::error::result::ScrapResult};

pub fn run(project_name: &str) -> ScrapResult<()> {
    InitCommand::new(project_name).run()
}
