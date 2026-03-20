use std::path::Path;

use annotate_snippets::{Level, Renderer, Snippet};
use colored::Colorize;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::lint::rule::{LintRuleName, LintWarning};
use crate::usecase::lint::usecase::LintUsecase;

pub fn run(project_path: Option<&Path>, rule_names: &[LintRuleName]) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let scraps_dir_name = config.scraps_dir.as_deref().unwrap_or(Path::new("scraps"));

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path)?;
    let usecase = LintUsecase::new();
    let warnings = usecase.execute(&scraps, rule_names)?;

    if warnings.is_empty() {
        return Ok(());
    }

    let renderer = Renderer::styled();

    for warning in &warnings {
        print_warning(warning, scraps_dir_name, &renderer);
    }

    let total = warnings.len();
    eprintln!(
        "{}",
        format!("warning: `scraps lint` generated {} warning(s)", total)
            .yellow()
            .bold()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use crate::usecase::lint::rule::LintRuleName;
    use rstest::rstest;

    #[rstest]
    fn run_succeeds_with_clean_project(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"[[b]]")
            .add_scrap("b.md", b"[[a]]");

        let result = run(Some(project.project_root.as_path()), &[]);
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_with_warnings(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("lonely.md", b"no links here");

        let result = run(Some(project.project_root.as_path()), &[]);
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let result = run(Some(project.project_root.as_path()), &[]);
        assert!(result.is_err());
    }

    #[rstest]
    fn run_succeeds_with_rule_filter(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("lonely.md", b"no links here");

        let result = run(
            Some(project.project_root.as_path()),
            &[LintRuleName::DeadEnd],
        );
        assert!(result.is_ok());
    }

    #[rstest]
    fn run_succeeds_with_empty_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"");

        let result = run(Some(project.project_root.as_path()), &[]);
        assert!(result.is_ok());
    }
}

fn print_warning(warning: &LintWarning, scraps_dir: &Path, renderer: &Renderer) {
    let file_path = scraps_dir.join(&warning.scrap_path);
    let file_path_str = file_path.to_string_lossy();
    let title = format!("{}: {}", warning.rule_name.as_str(), warning.message);

    match (warning.source.as_ref(), warning.span) {
        (Some(source), Some((start, end))) => {
            let message = Level::Warning.title(&title).snippet(
                Snippet::source(source)
                    .line_start(1)
                    .origin(&file_path_str)
                    .fold(true)
                    .annotation(Level::Warning.span(start..end)),
            );
            eprintln!("{}", renderer.render(message));
        }
        _ => {
            let message = Level::Warning.title(&title);
            eprintln!("{}", renderer.render(message));
            eprintln!(" {} {}", "-->".blue().bold(), file_path_str);
            eprintln!();
        }
    }
}
