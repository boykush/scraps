use crate::init::cmd::InitCommand;
use scraps_libs::{error::ScrapResult, git::GitCommandImpl};

pub fn run(project_name: &str) -> ScrapResult<()> {
    let git_command = GitCommandImpl::new();
    InitCommand::new(git_command).run(project_name)
}
