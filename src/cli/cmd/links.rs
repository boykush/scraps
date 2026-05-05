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
use crate::usecase::scrap::lookup_links::usecase::LookupScrapLinksUsecase;
use scraps_libs::model::context::Ctx;
use scraps_libs::model::title::Title;

#[derive(Debug, Serialize, Deserialize)]
struct LinksResponse {
    results: Vec<ScrapKeyJson>,
    count: usize,
}

pub fn run(
    title: &str,
    ctx: Option<&str>,
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
    let target_title = Title::from(title);
    let target_ctx = ctx.map(Ctx::from);

    let usecase = LookupScrapLinksUsecase::new();
    let results = usecase.execute(&scraps, &target_title, &target_ctx)?;

    let scrap_keys: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|r| ScrapKeyJson {
            title: r.title.to_string(),
            ctx: r.ctx.map(|c| c.to_string()),
        })
        .collect();

    if json {
        let count = scrap_keys.len();
        let response = LinksResponse {
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
    fn run_text_outputs_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nLinks to [[cargo]] and [[clippy]].")
            .add_scrap("cargo.md", b"# Cargo")
            .add_scrap("clippy.md", b"# Clippy");

        let mut buf = Vec::new();
        run(
            "rust",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cargo"));
        assert!(output.contains("clippy"));
    }

    #[rstest]
    fn run_json_outputs_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nLinks to [[cargo]] and [[clippy]].")
            .add_scrap("cargo.md", b"# Cargo")
            .add_scrap("clippy.md", b"# Clippy");

        let mut buf = Vec::new();
        run(
            "rust",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: LinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.results.len(), 2);

        let titles: Vec<&str> = response.results.iter().map(|r| r.title.as_str()).collect();
        assert!(titles.contains(&"cargo"));
        assert!(titles.contains(&"clippy"));
    }

    #[rstest]
    fn run_json_outputs_empty_for_no_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nNo links here.");

        let mut buf = Vec::new();
        run(
            "rust",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: LinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_with_ctx_resolves_target(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nLinks to [[Token]].")
            .add_scrap("Frontend/Auth.md", b"# Auth\n\nLinks to [[Form]].")
            .add_scrap("Token.md", b"# Token")
            .add_scrap("Form.md", b"# Form");

        let mut buf = Vec::new();
        run(
            "Auth",
            Some("Backend"),
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: LinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results[0].title, "Token");
    }

    #[rstest]
    fn run_errors_when_ctx_is_omitted_for_ctx_scoped_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nLinks to [[Token]].")
            .add_scrap("Token.md", b"# Token");

        let mut buf = Vec::new();
        let result = run(
            "Auth",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }

    #[rstest]
    fn run_errors_on_missing_title(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("rust.md", b"# Rust");

        let mut buf = Vec::new();
        let result = run(
            "nonexistent",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            "rust",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
