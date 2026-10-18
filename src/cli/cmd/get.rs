use std::io::Write;
use std::path::Path;

use crate::cli::config::scrap_config::ScrapConfig;
use crate::cli::json::scrap::{CodeBlockJson, HeadingJson, VerifiedJson};
use crate::cli::path_resolver::PathResolver;
use crate::cli::scrap_arg::scrap_key;
use crate::error::ScrapsResult;
use crate::ir::loader;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use scraps_libs::markdown::query::{body, code_blocks, heading_slug, headings, images, section};
use serde_json::{Map, Value};

const FIELD_KEY: &str = "key";
const FIELD_TITLE: &str = "title";
const FIELD_CTX: &str = "ctx";
const FIELD_BODY: &str = "body";
const FIELD_HEADINGS: &str = "headings";
const FIELD_CODE_BLOCKS: &str = "code_blocks";
const FIELD_IMAGES: &str = "images";
const FIELD_TYPE: &str = "type";
const FIELD_DESCRIPTION: &str = "description";
const FIELD_FRONTMATTER: &str = "frontmatter";
const FIELD_VERIFIED: &str = "verified";

const DEFAULT_FIELDS: &[&str] = &[FIELD_KEY, FIELD_TITLE, FIELD_CTX, FIELD_BODY];
const ALLOWED_FIELDS: &[&str] = &[
    FIELD_KEY,
    FIELD_TITLE,
    FIELD_CTX,
    FIELD_BODY,
    FIELD_HEADINGS,
    FIELD_CODE_BLOCKS,
    FIELD_IMAGES,
    FIELD_TYPE,
    FIELD_DESCRIPTION,
    FIELD_FRONTMATTER,
    FIELD_VERIFIED,
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

fn scoped_text<'a>(md_text: &'a str, heading: Option<&str>) -> ScrapsResult<&'a str> {
    match heading {
        None => Ok(md_text),
        Some(h) => section(md_text, &heading_slug(h))
            .ok_or_else(|| anyhow::anyhow!("Heading not found: '{}'", h)),
    }
}

pub fn run(
    key: &str,
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

    let scraps = loader::load_scraps(&scraps_dir_path, &exclude_dirs)?;
    let target_key = scrap_key(key, ctx);

    let usecase = GetScrapUsecase::new();
    let result = usecase.execute(&scraps, &target_key)?;

    // Structure is read from the scoped file text so its lines stay file
    // lines; only the body drops the frontmatter block.
    let scoped = scoped_text(&result.md_text, heading)?;
    let body_text = body(scoped);

    match json {
        None => {
            write!(writer, "{}", body_text)?;
        }
        Some(spec) => {
            let fields = parse_fields(spec)?;
            let mut out = Map::new();
            for field in fields {
                match field {
                    FIELD_KEY => {
                        out.insert(
                            field.to_string(),
                            Value::String(result.scrap.key.to_string()),
                        );
                    }
                    FIELD_TITLE => {
                        out.insert(field.to_string(), Value::String(result.scrap.title.clone()));
                    }
                    FIELD_CTX => {
                        out.insert(
                            field.to_string(),
                            result
                                .scrap
                                .ctx()
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
                            headings(scoped).into_iter().map(Into::into).collect();
                        out.insert(field.to_string(), serde_json::to_value(v)?);
                    }
                    FIELD_CODE_BLOCKS => {
                        let v: Vec<CodeBlockJson> =
                            code_blocks(scoped).into_iter().map(Into::into).collect();
                        out.insert(field.to_string(), serde_json::to_value(v)?);
                    }
                    FIELD_IMAGES => {
                        let v: Vec<String> = images(scoped)
                            .into_iter()
                            .map(|url| url.to_string())
                            .collect();
                        out.insert(field.to_string(), serde_json::to_value(v)?);
                    }
                    FIELD_TYPE => {
                        out.insert(
                            field.to_string(),
                            result
                                .scrap_type
                                .as_ref()
                                .map(|t| Value::String(t.clone()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    FIELD_DESCRIPTION => {
                        out.insert(
                            field.to_string(),
                            result
                                .description
                                .as_ref()
                                .map(|d| Value::String(d.clone()))
                                .unwrap_or(Value::Null),
                        );
                    }
                    FIELD_FRONTMATTER => {
                        out.insert(
                            field.to_string(),
                            result.frontmatter.clone().unwrap_or(Value::Null),
                        );
                    }
                    FIELD_VERIFIED => {
                        let v = VerifiedJson::new(&result.verifications, result.trust_tier);
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
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
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
    fn run_default_json_returns_key_title_ctx_body(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("Backend/Auth.md", b"# Auth\n\nBody");

        let output = run_get("Auth", Some("Backend"), None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();
        let obj = v.as_object().unwrap();

        assert_eq!(obj.len(), 4);
        assert_eq!(obj["key"], "Backend/Auth");
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
    fn run_json_field_projection_supports_images(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "Gallery.md",
            b"# Gallery\n\n![one](https://example.com/one.png)\n\n![bad](local.png)\n\n## More\n\n![two](https://example.com/two.jpg)\n",
        );

        let output = run_get("Gallery", None, None, Some("images"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(
            v["images"].as_array().unwrap(),
            &vec![
                Value::String("https://example.com/one.png".to_string()),
                Value::String("https://example.com/two.jpg".to_string()),
            ]
        );
    }

    #[rstest]
    fn run_json_images_respects_heading_scope(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "Gallery.md",
            b"# Gallery\n\n![one](https://example.com/one.png)\n\n## More\n\n![two](https://example.com/two.jpg)\n",
        );

        let output = run_get("Gallery", None, Some("More"), Some("images"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(
            v["images"].as_array().unwrap(),
            &vec![Value::String("https://example.com/two.jpg".to_string())]
        );
    }

    #[rstest]
    fn run_json_field_projection_supports_type(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n")
            .add_scrap("note.md", b"# Note\n");

        let typed = run_get("post", None, None, Some("title,type"), &project).unwrap();
        assert_eq!(typed.trim(), r#"{"title":"post","type":"Blog"}"#);

        let untyped = run_get("note", None, None, Some("title,type"), &project).unwrap();
        assert_eq!(untyped.trim(), r#"{"title":"note","type":null}"#);
    }

    #[rstest]
    fn run_default_json_leaves_the_type_out(#[from(temp_scrap_project)] project: TempScrapProject) {
        project
            .add_config(b"")
            .add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n");

        let output = run_get("post", None, None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert!(!v.as_object().unwrap().contains_key("type"));
    }

    #[rstest]
    fn run_json_field_projection_supports_description(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap(
                "orders.md",
                b"---\ndescription: One row per completed customer order.\n---\n\n# Orders\n",
            )
            .add_scrap("note.md", b"# Note\n");

        let described = run_get("orders", None, None, Some("title,description"), &project).unwrap();
        assert_eq!(
            described.trim(),
            r#"{"title":"orders","description":"One row per completed customer order."}"#
        );

        let plain = run_get("note", None, None, Some("title,description"), &project).unwrap();
        assert_eq!(plain.trim(), r#"{"title":"note","description":null}"#);
    }

    #[rstest]
    fn run_default_json_leaves_the_description_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project
            .add_config(b"")
            .add_scrap("orders.md", b"---\ndescription: Orders.\n---\n\n# Orders\n");

        let output = run_get("orders", None, None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert!(!v.as_object().unwrap().contains_key("description"));
    }

    const WITH_BLOCK: &[u8] =
        b"---\ntype: Blog\ntags: [a, b]\nowner: alice\n---\n\n\n# Post\n\ntext\n\n## More\n\nmore\n";

    #[rstest]
    fn run_json_body_leaves_the_frontmatter_block_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, Some("body"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(v["body"], "# Post\n\ntext\n\n## More\n\nmore\n");
    }

    #[rstest]
    fn run_text_leaves_the_frontmatter_block_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, None, &project).unwrap();

        assert_eq!(output, "# Post\n\ntext\n\n## More\n\nmore\n");
    }

    #[rstest]
    #[case::unclosed("---\ntype: Blog\n\n# Post\n")]
    #[case::malformed("---\ntype: [unclosed\n---\n\n# Post\n")]
    #[case::absent("# Post\n\ntext\n")]
    fn run_json_body_keeps_text_that_is_not_frontmatter(
        #[from(temp_scrap_project)] project: TempScrapProject,
        #[case] text: &str,
    ) {
        project
            .add_config(b"")
            .add_scrap("post.md", text.as_bytes());

        let output = run_get("post", None, None, Some("body,frontmatter"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(v["body"], text);
        assert!(v["frontmatter"].is_null());
    }

    #[rstest]
    fn run_json_field_projection_supports_frontmatter(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, Some("frontmatter"), &project).unwrap();

        assert_eq!(
            output.trim(),
            r#"{"frontmatter":{"type":"Blog","tags":["a","b"],"owner":"alice"}}"#
        );
    }

    #[rstest]
    fn run_json_field_projection_supports_verified(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap(
            "post.md",
            b"---\nverified:\n  - by: process:ci\n    at: 2026-10-16T00:00:00Z\n  - by: human:alice\n    at: 2026-10-17T09:00:00+09:00\n---\n\n# Post\n",
        );

        let output = run_get("post", None, None, Some("verified"), &project).unwrap();

        assert_eq!(
            output.trim(),
            r#"{"verified":{"tier":"human-reviewed","entries":[{"by":"human:alice","at":"2026-10-17T00:00:00Z"},{"by":"process:ci","at":"2026-10-16T00:00:00Z"}]}}"#
        );
    }

    #[rstest]
    fn run_json_verified_is_unverified_without_the_key(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, Some("verified"), &project).unwrap();

        assert_eq!(
            output.trim(),
            r#"{"verified":{"tier":"unverified","entries":[]}}"#
        );
    }

    #[rstest]
    fn run_default_json_leaves_the_frontmatter_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert!(!v.as_object().unwrap().contains_key("frontmatter"));
    }

    #[rstest]
    fn run_json_structure_keeps_file_line_numbers(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, None, Some("headings"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        let lines: Vec<u64> = v["headings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["line"].as_u64().unwrap())
            .collect();
        assert_eq!(lines, vec![8, 12]);
    }

    #[rstest]
    fn run_json_with_heading_is_the_section_as_before(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_config(b"").add_scrap("post.md", WITH_BLOCK);

        let output = run_get("post", None, Some("More"), Some("body"), &project).unwrap();
        let v: Value = serde_json::from_str(output.trim()).unwrap();

        assert_eq!(v["body"], "\nmore\n");
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Heading not found")
        );
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

    #[rstest]
    fn run_takes_a_key_and_returns_key_title_and_ctx(
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

        let by_key = run_get("tables/orders", None, None, Some(""), &project).unwrap();
        let v: Value = serde_json::from_str(by_key.trim()).unwrap();
        assert_eq!(v["key"], "tables/orders");
        assert_eq!(v["title"], "Customer Orders");
        assert_eq!(v["ctx"], "tables");

        let by_stem = run_get("orders", Some("tables"), None, Some(""), &project).unwrap();
        assert_eq!(by_stem, by_key);
    }

    #[rstest]
    fn run_does_not_find_a_scrap_by_its_frontmatter_title(
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

        assert!(run_get("Customer Orders", Some("tables"), None, None, &project).is_err());
        assert!(run_get("tables/Customer Orders", None, None, None, &project).is_err());
    }
}
