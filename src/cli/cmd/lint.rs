use std::path::Path;

use colored::Colorize;
use itertools::Itertools;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::lint::rule::LintWarning;
use crate::usecase::lint::usecase::LintUsecase;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let usecase = LintUsecase::new(&scraps_dir_path);

    let warnings = usecase.execute()?;

    if warnings.is_empty() {
        println!("{}", "No lint warnings found.".green());
        return Ok(());
    }

    let grouped = warnings.iter().into_group_map_by(|w| w.rule_name.as_str());

    let mut rule_names: Vec<&str> = grouped.keys().copied().collect();
    rule_names.sort();

    for rule_name in rule_names {
        let rule_warnings = &grouped[rule_name];
        println!("{}", format!("[{}]", rule_name).yellow().bold());
        for warning in rule_warnings {
            print_warning(warning);
        }
        println!();
    }

    let total = warnings.len();
    println!(
        "{}",
        format!("Found {} lint warning(s).", total).yellow().bold()
    );

    Ok(())
}

fn print_warning(warning: &LintWarning) {
    println!(
        "  {} {}",
        warning.scrap_title.bold(),
        format!("- {}", warning.message).dimmed()
    );

    if let (Some(source), Some((start, end))) = (&warning.source, warning.span) {
        // Find line number and show context
        let before = &source[..start];
        let line_num = before.matches('\n').count() + 1;
        let line_start = before.rfind('\n').map_or(0, |i| i + 1);

        // Find end of the line containing the span end
        let line_end = source[end..].find('\n').map_or(source.len(), |i| end + i);
        let line_content = &source[line_start..line_end];

        println!(
            "    {}  {}",
            format!("{}|", line_num).dimmed(),
            line_content
        );
    }
}
