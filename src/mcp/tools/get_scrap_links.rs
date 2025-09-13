use crate::mcp::tools::search_scraps::SearchResultResponse;
use crate::usecase::scrap::get_links::usecase::GetScrapLinksUsecase;
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
pub struct GetScrapLinksRequest {
    /// Title of the scrap to get links for
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct GetScrapLinksResponse {
    pub results: Vec<SearchResultResponse>,
    pub count: usize,
}

pub async fn get_scrap_links(
    scraps_dir: &PathBuf,
    base_url: &BaseUrl,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<GetScrapLinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create get scrap links usecase
    let get_links_usecase = GetScrapLinksUsecase::new(scraps_dir);

    // Execute get links
    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let results = get_links_usecase
        .execute(base_url, &title, &ctx)
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32004),
                format!("Get scrap links failed: {e}"),
                None,
            )
        })?;

    // Convert results to structured response
    let link_results: Vec<SearchResultResponse> = results
        .into_iter()
        .map(|result| SearchResultResponse {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            md_text: result.md_text,
        })
        .collect();

    let count = link_results.len();
    let response = GetScrapLinksResponse {
        results: link_results,
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
