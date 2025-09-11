use crate::usecase::search::usecase::SearchUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use scraps_libs::model::base_url::BaseUrl;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct SearchRequest {
    /// Query string to search for
    pub query: String,
    /// Maximum number of results to return (default: 100)
    pub num: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub title: String,
    pub ctx: Option<String>,
    pub md_text: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResultResponse>,
    pub count: usize,
}

pub async fn search_scraps(
    scraps_dir: &PathBuf,
    base_url: &BaseUrl,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<SearchRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create search usecase
    let search_usecase = SearchUsecase::new(scraps_dir);

    // Execute search
    let num = request.num.unwrap_or(100);
    let results = search_usecase
        .execute(base_url, &request.query, num)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Search failed: {e}"), None))?;

    // Convert results to structured response
    let search_results: Vec<SearchResultResponse> = results
        .into_iter()
        .map(|result| SearchResultResponse {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            md_text: result.md_text,
        })
        .collect();

    let count = search_results.len();
    let response = SearchResponse {
        results: search_results,
        count,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
