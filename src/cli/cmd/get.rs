use std::io::Write;
use std::path::Path;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::{CodeBlockJson, HeadingJson};
use crate::cli::path_resolver::PathResolver;
use crate::error::ScrapsResult;
use crate::input::file::read_scraps;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use scraps_libs::markdown::query::{code_blocks, heading_slug, headings, section};
use scraps_libs::model::context::Ctx;
use scraps_libs::model::title::Title;
use serde_json::{Map, Value};

const FIELD_TITLE: &str = "title";
const FIELD_CTX: &str = "ctx";
const FIELD_BODY: &str = "body";
const FIELD_HEADINGS: &str = "headings";
const FIELD_CODE_BLOCKS: &str = "code_blocks";

const DEFAULT_FIELDS: &[&str] = &[FIELD_TITLE, FIELD_CTX, FIELD_BODY];
const ALLOWED_FIELDS: &[&str] = &[
    FIELD_TITLE,
    FIELD_CTX,
    FIELD_BODY,
    FIELD_HEADINGS,
    FIELD_CODE_BLOCKS,
];

fn parse_fields(spec: &str) -> ScrapsResult<Vec<&'static str>> {
    let trimmed = spec.trim();
    if trimmed.is_empty() {
        return Ok(DEFAULT_FIELDS.to_vec());
    }
    let mut out = Vec::new();
    for raw in trimmed.split(',') {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let allowed = ALLOWED_FIELDS.iter().find(|f| **f == name).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown --json field '{}'. Allowed fields: {}",
                name,
                ALLOWED_FIELDS.join(", ")
            )
        })?;
        if !out.contains(allowed) {
            out.push(*allowed);
        }
    }
    if out.is_empty() {
        return Ok(DEFAULT_FIELDS.to_vec());
    }
    Ok(out)
}

fn scoped_body<'a>(md_text: &'a str, heading: Option<&str>) -> ScrapsResult<&'a str> {
    match heading {
        None => Ok(md_text),
        Some(h) => section(md_text, &heading_slug(h))
            .ok_or_else(|| anyhow::anyhow!("Heading not found: '{}'", h)),
    }
}

pub fn run(
    title: &str,
    ctx: Option<&str>,
    heading: Option<&str>,
    json: Option<&str>,
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

    let usecase = GetScrapUsecase::new();
    let result = usecase.execute(&scraps, &target_title, &target_ctx)?;

    let body_text = scoped_body(&result.md_text, heading)?;

    match json {
        None => {
            write!(writer, "{}", body_text)?;
        }
        Some(spec) => {
            let fields = parse_fields(spec)?;
            let mut out = Map::new();
            for field in fields {
                match field {
                    FIELD_TITLE => {
                        out.insert(field.to_string(), Value::String(result.title.to_string()));
                    }
                    FIELD_CTX => {
                        out.insert(
                            field.to_string(),
                            result
                                .ctx
                                .as_ref()
                                .map(|c| Value::String(c.to_string()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    FIELD_BODY => {
                        out.insert(field.to_string(), Value::String(body_text.to_string()));
                    }
                    FIELD_HEADINGS => {
                        let v: Vec<HeadingJson> =
                            headings(body_text).into_iter().map(Into::into).collect();
                        out.insert(field.to_string(), serde_json::to_value(v)?);
                    }
                    FIELD_CODE_BLOCKS => {
                        let v: Vec<CodeBlockJson> =
                            code_blocks(body_text).into_iter().map(Into::into).collect();
                        out.insert(field.to_string(), serde_json::to_value(v)?);
                    }
                    _ => unreachable!("validated by parse_fields"),
                }
            }
            writeln!(writer, "{}", serde_json::to_string(&Value::Object(out))?)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rstest::rstest;
    use serde_json::Value;

    fn run_get(
        title: &str,
        ctx: Option<&str>,
        heading: Option<&str>,
        json: Option<&str>,
        project: &TempScrapProject,
    ) -> ScrapsResult<String> {
        let mut buf = Vec::new();
        run(
            title,
            ctx,
            heading,
            json,
            Some(project.project_root.as_path()),
            &mut buf,
        )?;
        Ok(String::from_utf8(buf).unwrap())
    }

    #[rstest]
    fn run_text_outputs_md_body(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nBody");

        let output = run_get("rust", None, None, None, &project).unwrap();
        assert!(output.contains("# Rust"));
        assert!(output.contains("Body"));
    }

    #[rstest]
    fn run_default_json_returns_title_ctx_body(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nBody");

        let output = run_get("Auth", Some("Backend"), None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        let obj = v.as_object().unwrap();

        assert_eq!(obj.len(), 3);
        assert_eq!(obj["title"], "Auth");
        assert_eq!(obj["ctx"], "Backend");
        assert!(obj["body"].as_str().unwrap().contains("# Auth"));
        assert!(!obj.contains_key("headings"));
        assert!(!obj.contains_key("code_blocks"));
    }

    #[rstest]
    fn run_default_json_emits_null_ctx_when_absent(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("plain.md", b"# Plain\n\nbody\n");

        let output = run_get("plain", None, None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        assert!(v["ctx"].is_null());
    }

    #[rstest]
    fn run_json_with_field_projection_includes_only_requested(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("doc.md", b"# Doc\n\n```rust\nlet x = 1;\n```\n");

        let output = run_get("doc", None, None, Some("code_blocks"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        let obj = v.as_object().unwrap();

        assert_eq!(obj.len(), 1);
        let blocks = obj["code_blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["lang"], "rust");
    }

    #[rstest]
    fn run_json_field_projection_supports_headings(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Tokio.md", b"# Tokio\n\nintro\n\n## Scheduling\n\nbody\n");

        let output = run_get("Tokio", None, None, Some("title,headings"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        let obj = v.as_object().unwrap();

        assert_eq!(obj.len(), 2);
        assert_eq!(obj["title"], "Tokio");
        let h = obj["headings"].as_array().unwrap();
        assert_eq!(h.len(), 2);
    }

    #[rstest]
    fn run_json_unknown_field_errors(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("rust.md", b"# Rust\n\nBody");

        let result = run_get("rust", None, None, Some("links"), &project);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Unknown --json field"));
        assert!(msg.contains("links"));
    }

    #[rstest]
    fn run_text_with_heading_returns_section_only(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "Guide.md",
            b"# Guide\n\nintro\n\n## Install\n\ninstall body\n\n## Usage\n\nusage body\n",
        );

        let output = run_get("Guide", None, Some("Install"), None, &project).unwrap();
        assert!(output.contains("install body"));
        assert!(!output.contains("usage body"));
        assert!(!output.contains("intro"));
    }

    #[rstest]
    fn run_json_with_heading_scopes_body_and_structure(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "Guide.md",
            b"# Guide\n\nintro\n\n## Install\n\ninstall body\n\n```sh\nrun me\n```\n\n## Usage\n\nusage body\n",
        );

        let output = run_get(
            "Guide",
            None,
            Some("Install"),
            Some("body,code_blocks"),
            &project,
        )
        .unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        let obj = v.as_object().unwrap();

        let body = obj["body"].as_str().unwrap();
        assert!(body.contains("install body"));
        assert!(!body.contains("usage body"));

        let blocks = obj["code_blocks"].as_array().unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["lang"], "sh");
    }

    #[rstest]
    fn run_errors_when_heading_not_found(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("Guide.md", b"# Guide\n\n## Install\n\nbody\n");

        let result = run_get("Guide", None, Some("Missing"), None, &project);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Heading not found"));
    }

    #[rstest]
    fn run_errors_when_ctx_is_omitted_for_ctx_scoped_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nFrom Backend");

        let result = run_get("Auth", None, None, None, &project);
        assert!(result.is_err());
    }

    #[rstest]
    fn run_errors_on_missing_title(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_config(b"").add_scrap("rust.md", b"# Rust");

        let result = run_get("nonexistent", None, None, None, &project);
        assert!(result.is_err());
    }

    #[rstest]
    fn run_fails_without_config(#[from(temp_scrap_project)] project: TempScrapProject) {
        let result = run_get("rust", None, None, None, &project);
        assert!(result.is_err());
    }
}
