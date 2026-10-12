use std::io::Write;
use std::path::Path;

use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{Cell, CellAlignment, Table};

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap_type::TypeJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::usecase::scrap_type::list::usecase::ListTypeUsecase;

pub fn run(json: bool, project_path: Option<&Path>, writer: &mut impl Write) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir();
    let exclude_dirs = vec![
        path_resolver.static_dir(),
        path_resolver.output_dir(&config),
    ];

    let scraps = loader::load_scraps(&scraps_dir_path, &exclude_dirs)?;
    let usecase = ListTypeUsecase::new();

    let types = usecase.execute(&scraps)?;

    if json {
        let types_json: Vec<TypeJson> = types
            .into_iter()
            .map(|t| TypeJson {
                name: t.name,
                scraps_count: t.scraps_count,
            })
            .collect();
        writeln!(writer, "{}", serde_json::to_string(&types_json)?)?;
    } else {
        if types.is_empty() {
            return Ok(());
        }

        let mut table = Table::new();
        table.load_style(NOTHING);
        table.set_header(vec![Cell::new("Type".bold()), Cell::new("Scraps".bold())]);

        for t in &types {
            table.add_row(vec![
                Cell::new(&t.name),
                Cell::new(t.scraps_count).set_alignment(CellAlignment::Right),
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
    fn run_json_outputs_types_most_used_first(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n")
            .add_scrap("one.md", b"---\ntype: Book\n---\n\n# One\n")
            .add_scrap("two.md", b"---\ntype: Book\n---\n\n# Two\n")
            .add_scrap("note.md", b"# Note\n\n#[[Blog]]\n");

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output.trim(),
            r#"[{"type":"Book","scraps_count":2},{"type":"Blog","scraps_count":1}]"#
        );
    }

    #[rstest]
    fn run_json_outputs_empty_array_for_an_untyped_wiki(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("note.md", b"# Note\n");

        let mut buf = Vec::new();
        run(true, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        let types: Vec<TypeJson> = serde_json::from_str(output.trim()).unwrap();
        assert!(types.is_empty());
    }

    #[rstest]
    fn run_text_lists_types_with_counts(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n");

        let mut buf = Vec::new();
        run(false, Some(project.project_root.as_path()), &mut buf).unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Blog"));
        assert!(output.contains('1'));
    }

    #[rstest]
    fn run_text_empty_for_an_untyped_wiki(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("note.md", b"# Note\n");

        let mut buf = Vec::new();
        run(false, Some(project.project_root.as_path()), &mut buf).unwrap();

        assert!(buf.is_empty());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let mut buf = Vec::new();
        let result = run(false, Some(project.project_root.as_path()), &mut buf);
        assert!(result.is_err());
    }
}
