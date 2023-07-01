use crate::{
    init::cmd::InitCommand,
    libs::{error::result::ScrapResult, git::GitCommandImpl},
};

pub fn run(project_name: &str) -> ScrapResult<()> {
    let git_command = GitCommandImpl::new();
    InitCommand::new(project_name, Box::new(git_command)).run()
}
