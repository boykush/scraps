use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::init::usecase::InitUsecase;
use std::path::Path;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    InitUsecase::new().execute(path_resolver.project_root())
}
