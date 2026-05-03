use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::ScrapKeyJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::search::usecase::SearchUsecase;
use scraps_libs::search::engine::SearchLogic;

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<ScrapKeyJson>,
    count: usize,
}

pub fn run(
    query: &str,
    num: usize,
    logic: SearchLogic,
    json: bool,
    project_path: Option<&Path>,
    writer: &mut impl Write,
) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path)?;

    let usecase = SearchUsecase::new();
    let results = usecase.execute(&scraps, query, num, logic)?;

    let scrap_keys: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|r| ScrapKeyJson {
            title: r.title.to_string(),
            ctx: r.ctx.map(|c| c.to_string()),
        })
        .collect();

    if json {
        let count = scrap_keys.len();
        let response = SearchResponse {
            results: scrap_keys,
            count,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if scrap_keys.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(NOTHING);
        table.set_header(vec![Cell::new("Title".bold()), Cell::new("Context".bold())]);

        for key in &scrap_keys {
            table.add_row(vec![
                Cell::new(&key.title),
                Cell::new(key.ctx.as_deref().unwrap_or("")),
            ]);
        }
        writeln!(writer, "{table}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn run_text_outputs_titles(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust-basics.md", b"# Rust Basics\n\nLearning rust.")
            .add_scrap("ownership.md", b"# Ownership\n\nRust ownership rules.")
            .add_scrap("python.md", b"# Python\n\nA different language.");

        let mut buf = Vec::new();
        run(
            "rust",
            100,
            SearchLogic::Or,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("rust-basics"));
        assert!(output.contains("ownership"));
    }

    #[rstest]
    fn run_json_outputs_results(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust-basics.md", b"# Rust Basics\n\nLearning rust.")
            .add_scrap("ownership.md", b"# Ownership\n\nRust ownership rules.")
            .add_scrap("python.md", b"# Python\n\nA different language.");

        let mut buf = Vec::new();
        run(
            "rust",
            100,
            SearchLogic::Or,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: SearchResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, response.results.len());
        let titles: Vec<&str> = response.results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"rust-basics"));
        assert!(titles.contains(&"ownership"));
        assert!(!titles.contains(&"python"));
    }

    #[rstest]
    fn run_json_outputs_empty_for_no_match(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nA language.");

        let mut buf = Vec::new();
        run(
            "nonexistent-keyword-zzz",
            100,
            SearchLogic::Or,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: SearchResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_logic_and_requires_all_keywords(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust-python.md", b"# Rust Python\n\nrust and python.")
            .add_scrap("rust-only.md", b"# Rust Only\n\njust rust here.")
            .add_scrap("python-only.md", b"# Python Only\n\njust python here.");

        let mut buf = Vec::new();
        run(
            "rust python",
            100,
            SearchLogic::And,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: SearchResponse = serde_json::from_str(output.trim()).unwrap();
        let titles: Vec<&str> = response.results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"rust-python"));
        assert!(!titles.contains(&"rust-only"));
        assert!(!titles.contains(&"python-only"));
    }

    #[rstest]
    fn run_num_limits_results(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust1.md", b"# Rust1\n\nrust content.")
            .add_scrap("rust2.md", b"# Rust2\n\nrust content.")
            .add_scrap("rust3.md", b"# Rust3\n\nrust content.");

        let mut buf = Vec::new();
        run(
            "rust",
            2,
            SearchLogic::Or,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: SearchResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.results.len(), 2);
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            "rust",
            100,
            SearchLogic::Or,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
