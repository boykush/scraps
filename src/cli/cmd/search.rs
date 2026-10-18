use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::SearchHitJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::usecase::search::usecase::SearchUsecase;
use scraps_libs::search::engine::SearchLogic;

#[derive(Debug, Serialize, Deserialize)]
struct SearchResponse {
    results: Vec<SearchHitJson>,
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
    let scraps_dir_path = path_resolver.scraps_dir();
    let exclude_dirs = vec![
        path_resolver.static_dir(),
        path_resolver.output_dir(&config),
    ];

    let scraps = loader::load_scraps(&scraps_dir_path, &exclude_dirs)?;

    let usecase = SearchUsecase::new();
    let results = usecase.execute(&scraps, query, num, logic)?;

    let hits: Vec<SearchHitJson> = results
        .into_iter()
        .map(|r| SearchHitJson {
            scrap: (&r.scrap).into(),
            description: r.description,
        })
        .collect();

    if json {
        let count = hits.len();
        let response = SearchResponse {
            results: hits,
            count,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if hits.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_style(NOTHING);
        table.set_header(vec![
            Cell::new("Key".bold()),
            Cell::new("Title".bold()),
            Cell::new("Context".bold()),
        ]);

        for hit in &hits {
            table.add_row(vec![
                Cell::new(&hit.scrap.key),
                Cell::new(&hit.scrap.title),
                Cell::new(hit.scrap.ctx.as_deref().unwrap_or("")),
            ]);
        }
        writeln!(writer, "{table}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;

    #[rstest]
    fn run_text_outputs_titles(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust-basics.md", b"# Rust Basics\n\nLearning rust.")
            .add_scrap("ownership.md", b"# Ownership\n\n#[[Rust]] ownership rules.")
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
            .add_scrap("ownership.md", b"# Ownership\n\n#[[Rust]] ownership rules.")
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
        let titles: Vec<&str> = response
            .results
            .iter()
            .map(|r| r.scrap.title.as_str())
            .collect();
        assert!(titles.contains(&"rust-basics"));
        assert!(titles.contains(&"ownership"));
        assert!(!titles.contains(&"python"));
    }

    #[rstest]
    fn run_json_carries_a_description_only_where_there_is_one(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "orders.md",
            b"---\ndescription: One row per completed customer order.\n---\n\n# Orders\n",
        );

        let search = |query: &str| {
            let mut buf = Vec::new();
            run(
                query,
                100,
                SearchLogic::Or,
                true,
                Some(project.project_root.as_path()),
                &mut buf,
            )
            .unwrap();
            String::from_utf8(buf).unwrap()
        };

        assert_eq!(
            search("orders").trim(),
            r#"{"results":[{"key":"orders","title":"orders","ctx":null,"description":"One row per completed customer order."}],"count":1}"#
        );

        project.add_scrap("orders.md", b"# Orders\n");
        assert_eq!(
            search("orders").trim(),
            r#"{"results":[{"key":"orders","title":"orders","ctx":null}],"count":1}"#
        );
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
        let titles: Vec<&str> = response
            .results
            .iter()
            .map(|r| r.scrap.title.as_str())
            .collect();
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

    #[rstest]
    fn run_json_finds_a_scrap_by_its_frontmatter_title(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap(
                "tables/orders.md",
                b"---\ntitle: Customer Orders\n---\n\nOne row per order.\n",
            )
            .add_scrap(
                "metrics/revenue.md",
                b"See [[tables/orders]] and [[tables/orders|the orders table]].\n",
            );

        let mut buf = Vec::new();
        run(
            "Customer",
            100,
            SearchLogic::Or,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        assert_eq!(
            String::from_utf8(buf).unwrap().trim(),
            r#"{"results":[{"key":"tables/orders","title":"Customer Orders","ctx":"tables"}],"count":1}"#
        );
    }
}
