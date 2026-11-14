use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::SecondsFormat;
use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::ScrapRefJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::usecase::verified::usecase::{ConfirmerFilter, VerifiedUsecase};

#[derive(Debug, Serialize, Deserialize)]
struct VerifiedItemJson {
    scrap: ScrapRefJson,
    by: String,
    at: String,
    age_days: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifiedResponse {
    results: Vec<VerifiedItemJson>,
    count: usize,
}

pub fn run(
    confirmer: ConfirmerFilter,
    older_than_days: Option<u64>,
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

    let now_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let results = VerifiedUsecase::new().execute(&scraps, confirmer, older_than_days, now_ts)?;

    if json {
        let items: Vec<VerifiedItemJson> = results
            .into_iter()
            .map(|r| VerifiedItemJson {
                scrap: (&r.scrap).into(),
                by: r.by,
                at: r.at.to_rfc3339_opts(SecondsFormat::Secs, true),
                age_days: r.age_days,
            })
            .collect();
        let response = VerifiedResponse {
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
            Cell::new("Verified by".bold()),
            Cell::new("Days ago".bold()),
        ]);

        for r in &results {
            table.add_row(vec![
                Cell::new(r.scrap.key.to_string()),
                Cell::new(&r.by),
                Cell::new(r.age_days),
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

    const PAGE: &[u8] = b"---\nverified:\n  - by: human:alice\n    at: 2020-01-01T09:00:00+09:00\n  - by: claude-code/opus\n    at: 2021-01-01T00:00:00Z\n---\nbody\n";

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-05/example/EX-01
    #[rstest]
    fn run_json_names_the_scrap_the_confirmer_and_the_time(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("LLM/Agent.md", PAGE)
            .add_scrap("plain.md", b"body\n");

        let mut buf = Vec::new();
        run(
            ConfirmerFilter::Human,
            Some(180),
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: VerifiedResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        let item = &response.results[0];
        assert_eq!(item.scrap.key, "LLM/Agent");
        assert_eq!(item.scrap.ctx.as_deref(), Some("LLM"));
        assert_eq!(item.by, "human:alice");
        assert_eq!(item.at, "2020-01-01T00:00:00Z");
        assert!(item.age_days > 180);
    }

    #[rstest]
    fn run_json_by_machine_reads_the_other_clock(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("LLM/Agent.md", PAGE);

        let mut buf = Vec::new();
        run(
            ConfirmerFilter::Machine,
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: VerifiedResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.results[0].by, "claude-code/opus");
    }

    // livt:automates livt://mapping/renew-stale-understanding/rule/R-05/example/EX-02
    #[rstest]
    fn run_json_outputs_empty_when_nothing_is_confirmed(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("plain.md", b"body\n");

        let mut buf = Vec::new();
        run(
            ConfirmerFilter::Any,
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: VerifiedResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_text_lists_the_scrap_and_who_confirmed_it(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("LLM/Agent.md", PAGE);

        let mut buf = Vec::new();
        run(
            ConfirmerFilter::Any,
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("LLM/Agent"));
        assert!(output.contains("claude-code/opus"));
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            ConfirmerFilter::Any,
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
