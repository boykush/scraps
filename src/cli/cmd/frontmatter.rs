use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::frontmatter::usecase::FrontmatterUsecase;

#[derive(Debug, Serialize, Deserialize)]
struct FrontmatterScrapJson {
    title: String,
    ctx: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontmatterItemJson {
    scrap: FrontmatterScrapJson,
    frontmatter: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct FrontmatterResponse {
    results: Vec<FrontmatterItemJson>,
    count: usize,
}

fn scrap_label(title: &str, ctx: Option<&str>) -> String {
    match ctx {
        Some(c) if !c.is_empty() => format!("{c}/{title}"),
        _ => title.to_string(),
    }
}

/// Table cells hold one line, so a nested value is shown as compact JSON
/// rather than flattened into keys scraps would then have to name.
fn cell_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// A non-object frontmatter (a bare sequence, say) has no keys to spread over
/// rows, so the whole document occupies one.
fn rows(value: &Value) -> Vec<(String, String)> {
    match value {
        Value::Object(map) => map
            .iter()
            .map(|(key, value)| (key.clone(), cell_value(value)))
            .collect(),
        other => vec![(String::new(), other.to_string())],
    }
}

pub fn run(json: bool, project_path: Option<&Path>, writer: &mut impl Write) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let exclude_dirs = vec![
        path_resolver.static_dir(),
        path_resolver.output_dir(&config),
    ];

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path, &exclude_dirs)?;

    let usecase = FrontmatterUsecase::new();
    let results = usecase.execute(&scraps)?;

    if json {
        let items: Vec<FrontmatterItemJson> = results
            .into_iter()
            .map(|r| FrontmatterItemJson {
                scrap: FrontmatterScrapJson {
                    title: r.title.to_string(),
                    ctx: r.ctx.map(|c| c.to_string()),
                },
                frontmatter: r.frontmatter,
            })
            .collect();
        let response = FrontmatterResponse {
            count: items.len(),
            results: items,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if results.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_style(NOTHING);
        table.set_header(vec![
            Cell::new("Scrap".bold()),
            Cell::new("Key".bold()),
            Cell::new("Value".bold()),
        ]);

        for r in &results {
            let scrap = scrap_label(
                &r.title.to_string(),
                r.ctx.as_ref().map(|c| c.to_string()).as_deref(),
            );
            for (key, value) in rows(&r.frontmatter) {
                table.add_row(vec![Cell::new(&scrap), Cell::new(key), Cell::new(value)]);
            }
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
    use serde_json::json;

    #[rstest]
    fn run_text_lists_keys_per_scrap(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"---\nstatus: draft\nowner: alice\n---\n\n# A\n")
            .add_scrap("b.md", b"# B\n\nNo frontmatter.\n");

        let mut buf = Vec::new();
        run(false, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("status"));
        assert!(output.contains("draft"));
        assert!(output.contains("alice"));
        assert!(!output.contains("No frontmatter"));
    }

    #[rstest]
    fn run_json_outputs_results(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap(
            "Programming/Rust/borrowing.md",
            b"---\nstatus: draft\ntaxonomy:\n  - infra\n---\n\n# borrowing\n",
        );

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: FrontmatterResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        let item = &response.results[0];
        assert_eq!(item.scrap.title, "borrowing");
        assert_eq!(item.scrap.ctx.as_deref(), Some("Programming/Rust"));
        assert_eq!(
            item.frontmatter,
            json!({"status": "draft", "taxonomy": ["infra"]})
        );
    }

    #[rstest]
    fn run_json_outputs_empty_without_frontmatter(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"# A\n\nProse.\n");

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: FrontmatterResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_err());
    }
}
