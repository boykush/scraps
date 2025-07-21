use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::init::usecase::InitUsecase;
use scraps_libs::git::GitCommandImpl;
use std::path::Path;

pub fn run(project_name: &str, project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let base_dir = path_resolver.project_root();
    let project_dir = base_dir.join(project_name);
    let git_command = GitCommandImpl::new();
    InitUsecase::new(git_command).run(&project_dir)
}
