use crate::mcp::json::scrap::ScrapJson;
use crate::usecase::scrap::lookup_backlinks::usecase::LookupScrapBacklinksUsecase;
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
pub struct LookupScrapBacklinksRequest {
    /// Title of the scrap to get backlinks for
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapBacklinksResponse {
    pub results: Vec<ScrapJson>,
    pub count: usize,
}

pub async fn lookup_scrap_backlinks(
    scraps_dir: &PathBuf,
    base_url: &BaseUrl,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapBacklinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create get scrap backlinks usecase
    let get_backlinks_usecase = LookupScrapBacklinksUsecase::new(scraps_dir);

    // Execute get backlinks
    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let results = get_backlinks_usecase
        .execute(base_url, &title, &ctx)
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32006),
                format!("Get scrap backlinks failed: {e}"),
                None,
            )
        })?;

    // Convert results to structured response
    let scrap_jsons: Vec<ScrapJson> = results
        .into_iter()
        .map(|result| ScrapJson {
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            md_text: result.md_text,
        })
        .collect();

    let count = scrap_jsons.len();
    let response = LookupScrapBacklinksResponse {
        results: scrap_jsons,
        count,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32007),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
