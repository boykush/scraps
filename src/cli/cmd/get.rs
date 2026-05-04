use std::io::Write;
use std::path::Path;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::{CodeBlockJson, HeadingJson, ScrapJson};
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use scraps_libs::markdown::query::{code_blocks, headings};
use scraps_libs::model::context::Ctx;
use scraps_libs::model::title::Title;

pub fn run(
    title: &str,
    ctx: Option<&str>,
    json: bool,
    project_path: Option<&Path>,
    writer: &mut impl Write,
) -> ScrapsResult<()> {
    let path_resolver = PathResolver::new(project_path)?;
    let config = ScrapConfig::from_path(project_path)?;
    let scraps_dir_path = path_resolver.scraps_dir(&config);

    let scraps = read_scraps::to_all_scraps(&scraps_dir_path)?;
    let target_title = Title::from(title);
    let target_ctx = ctx.map(Ctx::from);

    let usecase = GetScrapUsecase::new();
    let result = usecase.execute(&scraps, &target_title, &target_ctx)?;

    if json {
        let headings_json: Vec<HeadingJson> = headings(&result.md_text)
            .into_iter()
            .map(Into::into)
            .collect();
        let code_blocks_json: Vec<CodeBlockJson> = code_blocks(&result.md_text)
            .into_iter()
            .map(Into::into)
            .collect();
        let scrap_json = ScrapJson {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            md_text: result.md_text,
            headings: headings_json,
            code_blocks: code_blocks_json,
        };
        writeln!(writer, "{}", serde_json::to_string(&scrap_json)?)?;
    } else {
        write!(writer, "{}", result.md_text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;

    #[rstest]
    fn run_text_outputs_md_body(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nBody");

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
        assert!(output.contains("# Rust"));
        assert!(output.contains("Body"));
    }

    #[rstest]
    fn run_json_outputs_scrap(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nBody");

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
        let scrap: ScrapJson = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(scrap.title, "Auth");
        assert_eq!(scrap.ctx.as_deref(), Some("Backend"));
        assert!(scrap.md_text.contains("# Auth"));
    }

    #[rstest]
    fn run_json_includes_headings_with_structure(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "Tokio.md",
            b"# Tokio\n\nintro\n\n## Scheduling\n\nbody\n\n### Work-stealing\n\ndetails\n",
        );

        let mut buf = Vec::new();
        run(
            "Tokio",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let scrap: ScrapJson = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(scrap.headings.len(), 3);

        assert_eq!(scrap.headings[0].level, 1);
        assert_eq!(scrap.headings[0].text, "Tokio");
        assert_eq!(scrap.headings[0].line, 1);
        assert!(scrap.headings[0].parent.is_none());

        assert_eq!(scrap.headings[1].level, 2);
        assert_eq!(scrap.headings[1].text, "Scheduling");
        assert_eq!(scrap.headings[1].parent.as_deref(), Some("Tokio"));

        assert_eq!(scrap.headings[2].level, 3);
        assert_eq!(scrap.headings[2].text, "Work-stealing");
        assert_eq!(scrap.headings[2].parent.as_deref(), Some("Scheduling"));
    }

    #[rstest]
    fn run_json_includes_fenced_code_blocks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap(
            "Snippets.md",
            b"# Snippets\n\n```rust\nlet x = 1;\n```\n\ntext\n\n```sh\ncargo build\n```\n",
        );

        let mut buf = Vec::new();
        run(
            "Snippets",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let scrap: ScrapJson = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(scrap.code_blocks.len(), 2);
        assert_eq!(scrap.code_blocks[0].lang.as_deref(), Some("rust"));
        assert_eq!(scrap.code_blocks[0].content, "let x = 1;\n");
        assert_eq!(scrap.code_blocks[1].lang.as_deref(), Some("sh"));
        assert_eq!(scrap.code_blocks[1].content, "cargo build\n");
    }

    #[rstest]
    fn run_json_emits_empty_arrays_for_plain_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("plain.md", b"just a paragraph\n");

        let mut buf = Vec::new();
        run(
            "plain",
            None,
            true,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        let scrap: ScrapJson = serde_json::from_str(output.trim()).unwrap();
        assert!(scrap.headings.is_empty());
        assert!(scrap.code_blocks.is_empty());
    }

    #[rstest]
    fn run_text_output_omits_structural_data(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("doc.md", b"# Doc\n\n```rust\nlet x = 1;\n```\n");

        let mut buf = Vec::new();
        run(
            "doc",
            None,
            false,
            Some(project.project_root.as_path()),
            &mut buf,
        )
        .unwrap();

        let output = String::from_utf8(buf).unwrap();
        assert!(!output.contains("\"headings\""));
        assert!(!output.contains("\"code_blocks\""));
    }

    #[rstest]
    fn run_errors_when_ctx_is_omitted_for_ctx_scoped_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nFrom Backend");

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
