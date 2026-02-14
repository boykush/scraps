use crate::mcp::json::scrap::ScrapKeyJson;
use crate::usecase::tag::lookup_backlinks::usecase::LookupTagBacklinksUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
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
    pub results: Vec<ScrapKeyJson>,
    pub count: usize,
}

pub async fn lookup_tag_backlinks(
    scraps_dir: &Path,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupTagBacklinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create tag backlinks usecase
    let lookup_usecase = LookupTagBacklinksUsecase::new(scraps_dir);

    // Execute lookup
    let tag_title = scraps_libs::model::title::Title::from(request.tag.as_str());

    let results = lookup_usecase.execute(&tag_title).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32008),
            format!("Lookup tag backlinks failed: {e}"),
            None,
        )
    })?;

    // Convert results to structured response
    let scrap_jsons: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|result| ScrapKeyJson {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
        })
        .collect();

    let count = scrap_jsons.len();
    let response = LookupTagBacklinksResponse {
        results: scrap_jsons,
        count,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32009),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
