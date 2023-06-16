use crate::{libs::error::result::ScrapResult, init::cmd::InitCommand};


pub fn run(project_name: &str) -> ScrapResult<()> {
    InitCommand::new(project_name).run()
}