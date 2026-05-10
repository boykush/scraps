use crate::input::file::read_scraps;
use crate::mcp::json::scrap::{CodeBlockJson, HeadingJson};
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use scraps_libs::markdown::query::{code_blocks, heading_slug, headings, section};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::path::Path;

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

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GetScrapRequest {
    /// Title of the scrap to get
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
    /// Optional heading text. When set, body/headings/code_blocks are scoped to that section.
    pub heading: Option<String>,
    /// Optional projection of fields to include. Defaults to ["title", "ctx", "body"].
    /// Allowed: "title", "ctx", "body", "headings", "code_blocks".
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
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let usecase = GetScrapUsecase::new();

    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let result = usecase
        .execute(&scraps, &title, &ctx)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Get scrap failed: {e}"), None))?;

    let body_text: &str = match request.heading.as_deref() {
        None => &result.md_text,
        Some(h) => section(&result.md_text, &heading_slug(h)).ok_or_else(|| {
            ErrorData::new(ErrorCode(-32004), format!("Heading not found: '{h}'"), None)
        })?,
    };

    let fields = resolve_fields(request.fields.as_deref())?;

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
                let v: Vec<HeadingJson> = headings(body_text).into_iter().map(Into::into).collect();
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
                    code_blocks(body_text).into_iter().map(Into::into).collect();
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
            _ => unreachable!("validated by resolve_fields"),
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&Value::Object(out)).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
