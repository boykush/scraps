use std::path::Path;

use scraps_libs::git::GitCommandImpl;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::output::build_renderer::BuildRendererImpl;
use crate::usecase::build::usecase::BuildUsecase;
use crate::usecase::progress::{Progress, Stage};

/// The build prints what it generated; a check generates nothing to report.
struct SilentProgress;

impl Progress for SilentProgress {
    fn start_stage(&self, _stage: &Stage) {}
    fn complete_stage(&self, _stage: &Stage, _count: &usize) {}
    fn end(&self) {}
}

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let build_config = config.to_build_config(None)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let static_dir_path = path_resolver.static_dir();
    let output_dir_path = path_resolver.output_dir(&config);

    // Commit dates only order and label pages, so a check never reads git.
    let exclude_dirs = vec![static_dir_path.clone(), output_dir_path];
    let (scraps_with_ts, readme_text) = loader::load_scraps_with_timestamps(
        &scraps_dir_path,
        &exclude_dirs,
        None::<GitCommandImpl>,
    )?;

    let renderer = BuildRendererImpl::discarding(&static_dir_path)?;
    let scrap_count = BuildUsecase::new().execute(
        &scraps_with_ts,
        &readme_text,
        &SilentProgress,
        &renderer,
        &build_config,
    )?;
    println!("Checked {scrap_count} scraps");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;

    const SSG_CONFIG: &[u8] = b"[ssg]\nbase_url = \"http://localhost:1112/\"\ntitle = \"Test\"";

    #[rstest]
    fn run_succeeds_without_writing_the_site(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(SSG_CONFIG)
            .add_scrap("test1.md", b"# header1\n")
            .add_scrap("test2.md", b"[[test1]] #[[tag]]\n");

        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_ok());

        let written = std::fs::read_dir(&project.output_dir).unwrap().count();
        assert_eq!(written, 0);
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_err());
    }

    #[rstest]
    fn run_fails_without_ssg_section(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("test1.md", b"# header1\n");

        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_err());
    }

    #[rstest]
    fn run_fails_on_a_broken_user_template(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(SSG_CONFIG)
            .add_scrap("test1.md", b"# header1\n")
            .add_static_file("index.html", b"{{ missing }}");

        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_err());
    }
}
