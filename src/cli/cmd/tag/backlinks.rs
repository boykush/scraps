use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::Serialize;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::ScrapKeyJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::tag::lookup_backlinks::usecase::LookupTagBacklinksUsecase;

#[derive(Debug, Serialize, serde::Deserialize)]
struct TagBacklinksResponse {
    results: Vec<ScrapKeyJson>,
    count: usize,
}

pub fn run(
    tag: &str,
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
    let usecase = LookupTagBacklinksUsecase::new();
    let tag_title = scraps_libs::model::title::Title::from(tag);
    let results = usecase.execute(&scraps, &tag_title)?;

    let scrap_keys: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|r| ScrapKeyJson {
            title: r.title.to_string(),
            ctx: r.ctx.map(|c| c.to_string()),
        })
        .collect();

    if json {
        let count = scrap_keys.len();
        let response = TagBacklinksResponse {
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
    fn run_json_outputs_backlinks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"#[[rust]]")
            .add_scrap("b.md", b"#[[rust]] #[[cli]]");

        let mut buf = Vec::new();
        run("rust", true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TagBacklinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.results.len(), 2);

        let titles: Vec<&str> = response.results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"a"));
        assert!(titles.contains(&"b"));
    }

    #[rstest]
    fn run_json_outputs_empty_for_unknown_tag(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("a.md", b"#[[rust]]");

        let mut buf = Vec::new();
        run(
            "nonexistent",
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TagBacklinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_text_succeeds(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("a.md", b"#[[rust]]")
            .add_scrap("b.md", b"#[[rust]]");

        let mut buf = Vec::new();
        let result = run(
            "rust",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("a"));
        assert!(output.contains("b"));
    }

    #[rstest]
    fn run_text_empty_for_no_backlinks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("a.md", b"#[[rust]]");

        let mut buf = Vec::new();
        let result = run(
            "nonexistent",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_ok());
        let output = String::from_utf8(buf).unwrap();
        assert!(output.is_empty());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            "rust",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
