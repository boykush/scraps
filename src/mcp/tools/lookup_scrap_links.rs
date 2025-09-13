use crate::mcp::json::scrap::ScrapJson;
use crate::usecase::scrap::lookup_links::usecase::LookupScrapLinksUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupScrapLinksRequest {
    /// Title of the scrap to get links for
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapLinksResponse {
    pub results: Vec<ScrapJson>,
    pub count: usize,
}

pub async fn lookup_scrap_links(
    scraps_dir: &PathBuf,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapLinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Create get scrap links usecase
    let get_links_usecase = LookupScrapLinksUsecase::new(scraps_dir);

    // Execute get links
    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let results = get_links_usecase.execute(&title, &ctx).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32004),
            format!("Get scrap links failed: {e}"),
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
    let response = LookupScrapLinksResponse {
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
