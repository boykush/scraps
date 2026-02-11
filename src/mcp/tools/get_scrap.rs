use crate::mcp::json::scrap::ScrapJson;
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GetScrapRequest {
    /// Title of the scrap to get
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

pub async fn get_scrap(
    scraps_dir: &PathBuf,
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<GetScrapRequest>,
) -> Result<CallToolResult, ErrorData> {
    let usecase = GetScrapUsecase::new(scraps_dir);

    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let result = usecase
        .execute(&title, &ctx)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Get scrap failed: {e}"), None))?;

    let scrap_json = ScrapJson {
        title: result.title.to_string(),
        ctx: result.ctx.map(|c| c.to_string()),
        md_text: result.md_text,
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string(&scrap_json).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
