use std::path::Path;
use std::process;

use colored::Colorize;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::usecase::lint::usecase::LintUsecase;

pub fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);
    let usecase = LintUsecase::new(&scraps_dir_path);

    let warnings = usecase.execute()?;

    if warnings.is_empty() {
        println!("{}", "No warnings found.".green());
        return Ok(());
    }

    for warning in &warnings {
        eprintln!(
            "{}",
            format!(
                "warning: implicit tag [[{}]] in \"{}\".",
                warning.broken_link, warning.scrap_title
            )
            .yellow()
        );
    }
    eprintln!(
        "{}",
        format!(
            "\nFound {} warning(s). Use #[[tag]] to explicitly mark as a tag.",
            warnings.len()
        )
        .yellow()
    );

    process::exit(1);
}
