use std::path::Path;

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

    for warning in &warnings {
        print_warning(warning, scraps_dir_name);
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

fn print_warning(warning: &LintWarning, scraps_dir: &Path) {
    let file_path = scraps_dir.join(&warning.scrap_path);
    let file_path_str = file_path.display();

    // warning[rule-name]: message
    eprintln!(
        "{}{}{}{} {}",
        "warning".yellow().bold(),
        "[".bold(),
        warning.rule_name.yellow().bold(),
        "]".bold(),
        format!(": {}", warning.message).bold()
    );

    match (warning.source.as_ref(), warning.span) {
        (Some(source), Some((start, end))) => {
            let (line, col) = warning.line_col().unwrap_or((1, 1));

            // --> file:line:col
            eprintln!(
                " {} {}",
                "-->".blue().bold(),
                format!("{}:{}:{}", file_path_str, line, col)
            );

            // source snippet
            let before = &source[..start];
            let line_start = before.rfind('\n').map_or(0, |i| i + 1);
            let line_end = source[end..].find('\n').map_or(source.len(), |i| end + i);
            let line_content = &source[line_start..line_end];
            let col_start = start - line_start;
            let span_len = end - start;
            let line_num_str = line.to_string();
            let pad = " ".repeat(line_num_str.len());

            eprintln!("{} {}", pad, "|".blue().bold());
            eprintln!(
                "{} {} {}",
                line_num_str.blue().bold(),
                "|".blue().bold(),
                line_content
            );
            eprintln!(
                "{} {} {}{}",
                pad,
                "|".blue().bold(),
                " ".repeat(col_start),
                "^".repeat(span_len).yellow().bold()
            );
            eprintln!("{} {}", pad, "|".blue().bold());
        }
        _ => {
            // --> file
            eprintln!(" {} {}", "-->".blue().bold(), file_path_str);
            eprintln!();
        }
    }
}
