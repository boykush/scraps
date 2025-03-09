use crate::init::cmd::InitCommand;
use scraps_libs::{error::ScrapsResult, git::GitCommandImpl};

pub fn run(project_name: &str) -> ScrapsResult<()> {
    let git_command = GitCommandImpl::new();
    InitCommand::new(git_command).run(project_name)
}
