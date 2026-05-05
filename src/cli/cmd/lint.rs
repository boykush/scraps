use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use annotate_snippets::{Level, Renderer, Snippet};
use colored::Colorize;
use scraps_libs::git::GitCommandImpl;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::lint::rule::{LintRule, LintRuleName, LintWarning};
use crate::usecase::lint::rules::stale_by_git::StaleByGitRule;
use crate::usecase::lint::usecase::LintUsecase;

pub fn run(project_path: Option<&Path>, rule_names: &[LintRuleName]) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let scraps_dir_name = config.scraps_dir.as_deref().unwrap_or(Path::new("scraps"));
    let scraps = read_scraps::to_all_scraps(&scraps_dir_path)?;

    // CLI `--rule X` overrides everything; otherwise default rules plus
    // opt-in rules whose config section enables them.
    let stale_config = config.lint.as_ref().and_then(|l| l.stale_by_git.as_ref());
    let effective_rules = if !rule_names.is_empty() {
        rule_names.to_vec()
    } else {
        let mut rules = LintRuleName::default_rules();
        if stale_config.is_some_and(|c| c.enabled) {
            rules.push(LintRuleName::StaleByGit);
        }
        rules
    };

    let extra_rules: Vec<Box<dyn LintRule>> = vec![Box::new(StaleByGitRule {
        git_command: GitCommandImpl::new(),
        scraps_dir: scraps_dir_path.clone(),
        threshold_days: stale_config.and_then(|c| c.threshold_days).unwrap_or(180),
        now_ts: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    })];

    let warnings = LintUsecase::new().execute(&scraps, &effective_rules, extra_rules)?;

    if warnings.is_empty() {
        return Ok(());
    }

    let renderer = Renderer::styled();
    for warning in &warnings {
        print_warning(warning, scraps_dir_name, &renderer);
    }
    eprintln!(
        "{}",
        format!(
            "warning: `scraps lint` generated {} warning(s)",
            warnings.len()
        )
        .yellow()
        .bold()
    );

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
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

    #[rstest]
    fn run_succeeds_with_stale_by_git_section(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"[lint.stale_by_git]\nthreshold_days = 30\n")
            .add_scrap("a.md", b"[[b]]")
            .add_scrap("b.md", b"[[a]]");

        // Without --rule, presence of [lint.stale_by_git] enables the opt-in
        // rule; outside a git repo it gracefully skips.
        let result = run(Some(project.project_root.as_path()), &[]);
        assert!(result.is_ok());
    }
}
