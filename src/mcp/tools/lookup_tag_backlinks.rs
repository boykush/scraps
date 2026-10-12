use crate::ir::loader;
use crate::mcp::json::scrap::ScrapRefJson;
use crate::usecase::tag::lookup_backlinks::usecase::LookupTagBacklinksUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupTagBacklinksRequest {
    /// Tag name to get backlinks for
    pub tag: String,
}

#[derive(Debug, Serialize)]
pub struct LookupTagBacklinksResponse {
    pub results: Vec<ScrapRefJson>,
    pub count: usize,
    pub next: String,
}

pub async fn lookup_tag_backlinks(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupTagBacklinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    // Create tag backlinks usecase
    let lookup_usecase = LookupTagBacklinksUsecase::new();

    // Execute lookup
    let requested_tag = scraps_libs::model::tag::Tag::from(request.tag.as_str());

    let results = lookup_usecase
        .execute(&scraps, &requested_tag)
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32008),
                format!("Lookup tag backlinks failed: {e}"),
                None,
            )
        })?;

    // Convert results to structured response
    let scrap_jsons: Vec<ScrapRefJson> = results
        .iter()
        .map(|result| (&result.scrap).into())
        .collect();

    let count = scrap_jsons.len();
    let next = if count == 0 {
        "No scraps under this tag. List available tags with list_tags."
    } else {
        "Read a result by passing its key back: get_scrap {key}."
    };
    let response = LookupTagBacklinksResponse {
        results: scrap_jsons,
        count,
        next: next.to_string(),
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32009),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
