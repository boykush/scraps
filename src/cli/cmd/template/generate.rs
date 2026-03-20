use std::path::Path;

use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use scraps_libs::model::title::Title;

use crate::{
    cli::config::scrap_config::ScrapConfig, usecase::template::generate::usecase::GenerateUsecase,
};

pub fn run(
    template_name: &str,
    scrap_title: &Option<Title>,
    project_path: Option<&Path>,
) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let templates_dir_path = path_resolver.templates_dir();
    let scraps_dir_path = path_resolver.scraps_dir(&config);

    let usecase = GenerateUsecase::new(&scraps_dir_path, &templates_dir_path);
    let timezone = config.timezone.unwrap_or(chrono_tz::UTC);
    usecase.execute(template_name, scrap_title, &timezone)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn run_succeeds_with_valid_template(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_template("daily.md", b"+++\ntitle = \"test_title\"\n+++\n\ncontent");

        let result = run("daily", &None, Some(project.project_root.as_path()));
        assert!(result.is_ok());
        assert!(project.scrap_path("test_title.md").exists());
    }

    #[rstest]
    fn run_fails_with_missing_template(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let result = run("nonexistent", &None, Some(project.project_root.as_path()));
        assert!(result.is_err());
    }

    #[rstest]
    fn run_succeeds_with_title_override(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_template("daily.md", b"+++\ntitle = \"default\"\n+++\n\ncontent");

        let title: Title = "custom_title".into();
        let result = run("daily", &Some(title), Some(project.project_root.as_path()));
        assert!(result.is_ok());
        assert!(project.scrap_path("custom_title.md").exists());
    }
}
