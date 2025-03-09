use crate::error::ScrapsResult;
use crate::usecase::init::cmd::InitCommand;
use scraps_libs::git::GitCommandImpl;

pub fn run(project_name: &str) -> ScrapsResult<()> {
    let git_command = GitCommandImpl::new();
    InitCommand::new(git_command).run(project_name)
}
