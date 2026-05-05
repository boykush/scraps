use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::todo::usecase::{StatusFilter, TodoUsecase};
use scraps_libs::markdown::query::TaskStatus;

#[derive(Debug, Serialize, Deserialize)]
struct TodoScrapJson {
    title: String,
    ctx: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoItemJson {
    scrap: TodoScrapJson,
    status: String,
    text: String,
    line: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct TodoResponse {
    results: Vec<TodoItemJson>,
    count: usize,
}

fn status_label(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Open => "open",
        TaskStatus::Done => "done",
        TaskStatus::Deferred => "deferred",
    }
}

fn scrap_label(title: &str, ctx: Option<&str>) -> String {
    match ctx {
        Some(c) if !c.is_empty() => format!("{c}/{title}"),
        _ => title.to_string(),
    }
}

pub fn run(
    status: StatusFilter,
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

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path, &exclude_dirs)?;

    let usecase = TodoUsecase::new();
    let results = usecase.execute(&scraps, status)?;

    if json {
        let items: Vec<TodoItemJson> = results
            .into_iter()
            .map(|r| TodoItemJson {
                scrap: TodoScrapJson {
                    title: r.title.to_string(),
                    ctx: r.ctx.map(|c| c.to_string()),
                },
                status: status_label(&r.status).to_string(),
                text: r.text,
                line: r.line,
            })
            .collect();
        let response = TodoResponse {
            count: items.len(),
            results: items,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if results.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(NOTHING);
        table.set_header(vec![
            Cell::new("Scrap".bold()),
            Cell::new("Status".bold()),
            Cell::new("Task".bold()),
        ]);

        for r in &results {
            let scrap = scrap_label(
                &r.title.to_string(),
                r.ctx.as_ref().map(|c| c.to_string()).as_deref(),
            );
            table.add_row(vec![
                Cell::new(scrap),
                Cell::new(status_label(&r.status)),
                Cell::new(&r.text),
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
    fn run_text_lists_open_tasks_by_default(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap(
                "borrowing.md",
                b"# borrowing\n\n- [ ] implement Drop for X\n- [x] write tests\n",
            )
            .add_scrap("python.md", b"# python\n\n- [-] write up zen\n");

        let mut buf = Vec::new();
        run(
            StatusFilter::Open,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("implement Drop for X"));
        assert!(!output.contains("write tests"));
        assert!(!output.contains("write up zen"));
    }

    #[rstest]
    fn run_json_outputs_results(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap(
            "Programming/Rust/borrowing.md",
            b"# borrowing\n\n- [ ] implement Drop for X\n",
        );

        let mut buf = Vec::new();
        run(
            StatusFilter::Open,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TodoResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results.len(), 1);
        let item = &response.results[0];
        assert_eq!(item.scrap.title, "borrowing");
        assert_eq!(item.scrap.ctx.as_deref(), Some("Programming/Rust"));
        assert_eq!(item.status, "open");
        assert_eq!(item.text, "implement Drop for X");
        assert_eq!(item.line, 3);
    }

    #[rstest]
    fn run_json_filters_by_status_done(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"- [ ] open\n- [x] done\n- [-] deferred\n");

        let mut buf = Vec::new();
        run(
            StatusFilter::Done,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TodoResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results[0].status, "done");
        assert_eq!(response.results[0].text, "done");
    }

    #[rstest]
    fn run_json_status_all_returns_all(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"- [ ] open\n- [x] done\n- [-] deferred\n");

        let mut buf = Vec::new();
        run(
            StatusFilter::All,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TodoResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 3);
    }

    #[rstest]
    fn run_json_outputs_empty_for_no_tasks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"# Just a heading.\n");

        let mut buf = Vec::new();
        run(
            StatusFilter::Open,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TodoResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            StatusFilter::Open,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
