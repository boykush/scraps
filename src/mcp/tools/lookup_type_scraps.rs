use crate::ir::loader;
use crate::mcp::json::scrap::ScrapRefJson;
use crate::usecase::scrap_type::lookup_scraps::usecase::LookupTypeScrapsUsecase;
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
pub struct LookupTypeScrapsRequest {
    /// Type to get the scraps of, exactly as list_types returns it (case-sensitive)
    pub r#type: String,
}

#[derive(Debug, Serialize)]
pub struct LookupTypeScrapsResponse {
    pub results: Vec<ScrapRefJson>,
    pub count: usize,
    pub next: String,
}

pub async fn lookup_type_scraps(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupTypeScrapsRequest>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let lookup_usecase = LookupTypeScrapsUsecase::new();

    let results = lookup_usecase
        .execute(&scraps, &request.r#type)
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32008),
                format!("Lookup type scraps failed: {e}"),
                None,
            )
        })?;

    let scrap_jsons: Vec<ScrapRefJson> = results
        .iter()
        .map(|result| (&result.scrap).into())
        .collect();

    let count = scrap_jsons.len();
    let next = if count == 0 {
        "No scraps of this type. List the types in use with list_types."
    } else {
        "Read a result by passing its key back: get_scrap {key}."
    };
    let response = LookupTypeScrapsResponse {
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
