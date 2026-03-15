use std::path::Path;

use annotate_snippets::{Level, Renderer, Snippet};
use colored::Colorize;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::lint::rule::LintWarning;
use crate::usecase::lint::usecase::LintUsecase;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let scraps_dir_name = config.scraps_dir.as_deref().unwrap_or(Path::new("scraps"));
    let usecase = LintUsecase::new(&scraps_dir_path);

    let warnings = usecase.execute()?;

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
