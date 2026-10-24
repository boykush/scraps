use crate::ir::loader;
use crate::mcp::json::scrap::{
    CodeBlockJson, GeneratedJson, HeadingJson, SourceJson, VerifiedJson,
};
use crate::mcp::scrap_target;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use scraps_libs::markdown::query::{body, code_blocks, heading_slug, headings, section};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::Path;

const FIELD_KEY: &str = "key";
const FIELD_TITLE: &str = "title";
const FIELD_CTX: &str = "ctx";
const FIELD_BODY: &str = "body";
const FIELD_HEADINGS: &str = "headings";
const FIELD_CODE_BLOCKS: &str = "code_blocks";
const FIELD_TYPE: &str = "type";
const FIELD_DESCRIPTION: &str = "description";
const FIELD_GENERATED: &str = "generated";
const FIELD_VERIFIED: &str = "verified";
const FIELD_SOURCES: &str = "sources";
const FIELD_RESOURCE: &str = "resource";

const DEFAULT_FIELDS: &[&str] = &[FIELD_KEY, FIELD_TITLE, FIELD_CTX, FIELD_BODY];
const ALLOWED_FIELDS: &[&str] = &[
    FIELD_KEY,
    FIELD_TITLE,
    FIELD_CTX,
    FIELD_BODY,
    FIELD_HEADINGS,
    FIELD_CODE_BLOCKS,
    FIELD_TYPE,
    FIELD_DESCRIPTION,
    FIELD_GENERATED,
    FIELD_VERIFIED,
    FIELD_SOURCES,
    FIELD_RESOURCE,
];

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GetScrapRequest {
    /// Key of the scrap to get: its path under the wiki root without `.md`, e.g. "tables/orders". Pass the `key` of a result back.
    pub key: Option<String>,
    /// Still accepted in place of `key`: the scrap's file stem (not its frontmatter title).
    pub title: Option<String>,
    /// Still accepted with `title`: the context (folder path) of the scrap.
    pub ctx: Option<String>,
    /// Optional heading text. When set, body/headings/code_blocks are scoped to that section.
    pub heading: Option<String>,
    /// Optional projection of fields to include. Defaults to ["key", "title", "ctx", "body"].
    /// Allowed: "key", "title", "ctx", "body", "headings", "code_blocks", "type", "description",
    /// "generated", "verified", "sources", "resource".
    pub fields: Option<Vec<String>>,
}

fn resolve_fields(spec: Option<&[String]>) -> Result<Vec<&'static str>, ErrorData> {
    let Some(spec) = spec else {
        return Ok(DEFAULT_FIELDS.to_vec());
    };
    if spec.is_empty() {
        return Ok(DEFAULT_FIELDS.to_vec());
    }
    let mut out: Vec<&'static str> = Vec::new();
    for raw in spec {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }
        let allowed = ALLOWED_FIELDS.iter().find(|f| **f == name).ok_or_else(|| {
            ErrorData::new(
                ErrorCode(-32602),
                format!(
                    "Unknown field '{}'. Allowed fields: {}",
                    name,
                    ALLOWED_FIELDS.join(", ")
                ),
                None,
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

pub async fn get_scrap(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<GetScrapRequest>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let usecase = GetScrapUsecase::new();

    let key = scrap_target::resolve(
        request.key.as_deref(),
        request.title.as_deref(),
        request.ctx.as_deref(),
    )?;

    let result = usecase
        .execute(&scraps, &key)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Get scrap failed: {e}"), None))?;

    // Structure is read from the scoped file text so its lines stay file
    // lines; only the body drops the frontmatter block.
    let scoped: &str = match request.heading.as_deref() {
        None => &result.md_text,
        Some(h) => section(&result.md_text, &heading_slug(h)).ok_or_else(|| {
            ErrorData::new(ErrorCode(-32004), format!("Heading not found: '{h}'"), None)
        })?,
    };

    let body_text = body(scoped);

    let fields = resolve_fields(request.fields.as_deref())?;

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
                let v: Vec<HeadingJson> = headings(scoped).into_iter().map(Into::into).collect();
                out.insert(
                    field.to_string(),
                    serde_json::to_value(v).map_err(|e| {
                        ErrorData::new(
                            ErrorCode(-32005),
                            format!("JSON serialization failed: {e}"),
                            None,
                        )
                    })?,
                );
            }
            FIELD_CODE_BLOCKS => {
                let v: Vec<CodeBlockJson> =
                    code_blocks(scoped).into_iter().map(Into::into).collect();
                out.insert(
                    field.to_string(),
                    serde_json::to_value(v).map_err(|e| {
                        ErrorData::new(
                            ErrorCode(-32005),
                            format!("JSON serialization failed: {e}"),
                            None,
                        )
                    })?,
                );
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
            FIELD_GENERATED => {
                let v = result.generated.as_ref().map(GeneratedJson::from);
                out.insert(
                    field.to_string(),
                    serde_json::to_value(v).map_err(|e| {
                        ErrorData::new(
                            ErrorCode(-32005),
                            format!("JSON serialization failed: {e}"),
                            None,
                        )
                    })?,
                );
            }
            FIELD_VERIFIED => {
                let v = VerifiedJson::new(&result.verifications, result.trust_tier);
                out.insert(
                    field.to_string(),
                    serde_json::to_value(v).map_err(|e| {
                        ErrorData::new(
                            ErrorCode(-32005),
                            format!("JSON serialization failed: {e}"),
                            None,
                        )
                    })?,
                );
            }
            FIELD_RESOURCE => {
                out.insert(
                    field.to_string(),
                    result
                        .resource
                        .clone()
                        .map(Value::String)
                        .unwrap_or(Value::Null),
                );
            }
            FIELD_SOURCES => {
                let sources: Vec<SourceJson> =
                    result.sources.iter().map(SourceJson::from).collect();
                out.insert(
                    field.to_string(),
                    serde_json::to_value(sources).map_err(|e| {
                        ErrorData::new(
                            ErrorCode(-32005),
                            format!("JSON serialization failed: {e}"),
                            None,
                        )
                    })?,
                );
            }
            _ => unreachable!("validated by resolve_fields"),
        }
    }
    out.insert(
        "next".to_string(),
        Value::String(
            "Traverse onward with lookup_scrap_links or lookup_scrap_backlinks {key}, or see everything around this scrap in one call with lookup_scrap_neighborhood {key}."
                .to_string(),
        ),
    );

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&Value::Object(out)).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
