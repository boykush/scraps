use std::path::Path;

use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;

use crate::usecase::template::list::usecase::ListUsecase;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let templates_dir_path = path_resolver.templates_dir();

    let usecase = ListUsecase::new(&templates_dir_path);
    let template_names = usecase.execute()?;

    for template_name in template_names {
        println!("{template_name}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn run_succeeds_with_templates(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_template("daily.md", b"+++\ntitle = \"daily\"\n+++\n\ncontent");

        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_with_no_templates(#[from(temp_scrap_project)] project: TempScrapProject) {
        let result = run(Some(project.project_root.as_path()));
        assert!(result.is_ok());
    }
}
