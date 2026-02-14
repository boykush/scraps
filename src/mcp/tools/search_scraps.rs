use crate::mcp::json::scrap::ScrapKeyJson;
use crate::usecase::search::usecase::SearchUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Search logic for combining multiple keywords
#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SearchLogic {
    /// All keywords must match
    And,
    /// Any keyword can match (default)
    #[default]
    Or,
}

impl From<SearchLogic> for scraps_libs::search::engine::SearchLogic {
    fn from(logic: SearchLogic) -> Self {
        match logic {
            SearchLogic::And => scraps_libs::search::engine::SearchLogic::And,
            SearchLogic::Or => scraps_libs::search::engine::SearchLogic::Or,
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct SearchRequest {
    /// Query string to search for
    pub query: String,
    /// Maximum number of results to return (default: 100)
    pub num: Option<usize>,
    /// Search logic: "and" (default, all keywords must match) or "or" (any keyword matches)
    pub logic: Option<SearchLogic>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<ScrapKeyJson>,
    pub count: usize,
}

pub async fn search_scraps(
    scraps_dir: &Path,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<SearchRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create search usecase
    let search_usecase = SearchUsecase::new(scraps_dir);

    // Execute search
    let num = request.num.unwrap_or(100);
    let logic = request.logic.unwrap_or_default().into();
    let results = search_usecase
        .execute(&request.query, num, logic)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Search failed: {e}"), None))?;

    // Convert results to structured response
    let scrap_jsons: Vec<ScrapKeyJson> = results
        .into_iter()
        .map(|result| ScrapKeyJson {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
        })
        .collect();

    let count = scrap_jsons.len();
    let response = SearchResponse {
        results: scrap_jsons,
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
