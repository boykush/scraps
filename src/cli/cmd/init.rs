use crate::init::cmd::InitCommand;
use crate::libs::git::GitCommandImpl;
use scraps_libs::error::ScrapResult;

pub fn run(project_name: &str) -> ScrapResult<()> {
    let git_command = GitCommandImpl::new();
    InitCommand::new(git_command).run(project_name)
}
