use crate::usecase::search::usecase::SearchUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use scraps_libs::model::base_url::BaseUrl;
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct SearchRequest {
    /// Query string to search for
    pub query: String,
    /// Maximum number of results to return (default: 100)
    pub num: Option<usize>,
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

    // Convert results to JSON
    let results_json = results
        .into_iter()
        .map(|result| {
            json!({
                "title": result.title,
                "md_text": result.md_text
            })
        })
        .collect::<Vec<_>>();

    let response = json!({
        "results": results_json,
        "count": results_json.len()
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
