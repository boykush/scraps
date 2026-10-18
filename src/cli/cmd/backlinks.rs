use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, Table};
use serde::{Deserialize, Serialize};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::{LinkKindJson, ScrapRefJson};
use crate::cli::path_resolver::PathResolver;
use crate::cli::scrap_arg::scrap_key;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::usecase::scrap::lookup_backlinks::usecase::LookupScrapBacklinksUsecase;

#[derive(Debug, Serialize, Deserialize)]
struct BacklinkJson {
    #[serde(flatten)]
    scrap: ScrapRefJson,
    kinds: Vec<LinkKindJson>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BacklinksResponse {
    results: Vec<BacklinkJson>,
    count: usize,
}

pub fn run(
    key: &str,
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

    let scraps = loader::load_scraps(&scraps_dir_path, &exclude_dirs)?;
    let target_key = scrap_key(key, ctx);

    let usecase = LookupScrapBacklinksUsecase::new();
    let results = usecase.execute(&scraps, &target_key)?;

    let scrap_refs: Vec<BacklinkJson> = results
        .iter()
        .map(|r| BacklinkJson {
            scrap: (&r.scrap).into(),
            kinds: r.kinds.iter().copied().map(LinkKindJson::from).collect(),
        })
        .collect();

    if json {
        let count = scrap_refs.len();
        let response = BacklinksResponse {
            results: scrap_refs,
            count,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if scrap_refs.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_style(NOTHING);
        table.set_header(vec![
            Cell::new("Key".bold()),
            Cell::new("Title".bold()),
            Cell::new("Context".bold()),
            Cell::new("Kinds".bold()),
        ]);

        for backlink in &scrap_refs {
            let scrap = &backlink.scrap;
            let kinds: Vec<&str> = backlink.kinds.iter().map(LinkKindJson::as_str).collect();
            table.add_row(vec![
                Cell::new(&scrap.key),
                Cell::new(&scrap.title),
                Cell::new(scrap.ctx.as_deref().unwrap_or("")),
                Cell::new(kinds.join(",")),
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
    fn run_text_outputs_backlinks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nLinks to [[cargo]].")
            .add_scrap("clippy.md", b"# Clippy\n\nAlso uses [[cargo]].")
            .add_scrap("cargo.md", b"# Cargo");

        let mut buf = Vec::new();
        run(
            "cargo",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("rust"));
        assert!(output.contains("clippy"));
    }

    #[rstest]
    fn run_json_outputs_backlinks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nLinks to [[cargo]].")
            .add_scrap("clippy.md", b"# Clippy\n\nAlso uses [[cargo]].")
            .add_scrap("cargo.md", b"# Cargo");

        let mut buf = Vec::new();
        run(
            "cargo",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: BacklinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.results.len(), 2);

        let titles: Vec<&str> = response
            .results
            .iter()
            .map(|r| r.scrap.title.as_str())
            .collect();
        assert!(titles.contains(&"rust"));
        assert!(titles.contains(&"clippy"));
    }

    #[rstest]
    fn run_json_outputs_empty_for_no_backlinks(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("cargo.md", b"# Cargo\n\nNo one links here.");

        let mut buf = Vec::new();
        run(
            "cargo",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: BacklinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_with_ctx_resolves_target(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap(
                "Backend/Server.md",
                b"# Server\n\nLinks to [[Backend/Auth]].",
            )
            .add_scrap(
                "Frontend/Login.md",
                b"# Login\n\nLinks to [[Frontend/Auth]].",
            )
            .add_scrap("Backend/Auth.md", b"# Auth\n\nBackend auth")
            .add_scrap("Frontend/Auth.md", b"# Auth\n\nFrontend auth");

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
        let response: BacklinksResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 1);
        assert_eq!(response.results[0].scrap.title, "Server");
    }

    #[rstest]
    fn run_errors_when_ctx_is_omitted_for_ctx_scoped_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap(
                "Backend/Server.md",
                b"# Server\n\nLinks to [[Backend/Auth]].",
            )
            .add_scrap("Backend/Auth.md", b"# Auth");

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

    #[rstest]
    fn run_json_takes_a_key_and_names_each_backlink_by_key(
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

        let backlinks = |key: &str, ctx: Option<&str>| {
            let mut buf = Vec::new();
            run(
                key,
                ctx,
                true,
                Some(project.project_root.as_path()),
                &mut buf,
            )
            .unwrap();
            String::from_utf8(buf).unwrap()
        };

        let by_key = backlinks("tables/orders", None);
        assert_eq!(
            by_key.trim(),
            r#"{"results":[{"key":"metrics/revenue","title":"revenue","ctx":"metrics","kinds":["link"]}],"count":1}"#
        );
        assert_eq!(backlinks("orders", Some("tables")), by_key);
    }
}
