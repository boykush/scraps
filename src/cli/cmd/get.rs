use std::io::Write;
use std::path::Path;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::ScrapJson;
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
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
        let scrap_json = ScrapJson {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            md_text: result.md_text,
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
