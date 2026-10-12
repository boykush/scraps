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
use crate::ir::loader;
use crate::usecase::scrap_type::lookup_scraps::usecase::LookupTypeScrapsUsecase;

#[derive(Debug, Serialize, serde::Deserialize)]
struct TypeScrapsResponse {
    results: Vec<ScrapKeyJson>,
    count: usize,
}

pub fn run(
    type_name: &str,
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
    let usecase = LookupTypeScrapsUsecase::new();
    let results = usecase.execute(&scraps, type_name)?;

    let scrap_keys: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|r| ScrapKeyJson {
            title: r.title.to_string(),
            ctx: r.ctx.map(|c| c.to_string()),
        })
        .collect();

    if json {
        let count = scrap_keys.len();
        let response = TypeScrapsResponse {
            results: scrap_keys,
            count,
        };
        writeln!(writer, "{}", serde_json::to_string(&response)?)?;
    } else {
        if scrap_keys.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_style(NOTHING);
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
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rstest::rstest;

    #[rstest]
    fn run_json_outputs_the_scraps_of_the_type(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n")
            .add_scrap("Posts/other.md", b"---\ntype: Blog\n---\n\n# Other\n")
            .add_scrap("summary.md", b"---\ntype: Book\n---\n\n# Summary\n")
            .add_scrap("note.md", b"# Note\n");

        let mut buf = Vec::new();
        run("Blog", true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output.trim(),
            r#"{"results":[{"title":"post","ctx":null},{"title":"other","ctx":"Posts"}],"count":2}"#
        );
    }

    #[rstest]
    fn run_json_outputs_empty_for_unknown_type(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n");

        let mut buf = Vec::new();
        run(
            "Playbook",
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let response: TypeScrapsResponse = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(response.count, 0);
        assert!(response.results.is_empty());
    }

    #[rstest]
    fn run_text_succeeds(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n")
            .add_scrap("summary.md", b"---\ntype: Book\n---\n\n# Summary\n");

        let mut buf = Vec::new();
        run(
            "Blog",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("post"));
        assert!(!output.contains("summary"));
    }

    #[rstest]
    fn run_text_empty_for_unknown_type(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("note.md", b"# Note\n");

        let mut buf = Vec::new();
        run(
            "Blog",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        assert!(buf.is_empty());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(
            "Blog",
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        );
        assert!(result.is_err());
    }
}
