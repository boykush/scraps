use crate::input::file::read_scraps;
use crate::mcp::json::scrap::{CodeBlockJson, HeadingJson, ScrapJson};
use crate::usecase::scrap::get::usecase::GetScrapUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use scraps_libs::markdown::query::{code_blocks, headings};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct GetScrapRequest {
    /// Title of the scrap to get
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

pub async fn get_scrap(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<GetScrapRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let usecase = GetScrapUsecase::new();

    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let result = usecase
        .execute(&scraps, &title, &ctx)
        .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Get scrap failed: {e}"), None))?;

    let headings_json: Vec<HeadingJson> = headings(&result.md_text)
        .into_iter()
        .map(Into::into)
        .collect();
    let code_blocks_json: Vec<CodeBlockJson> = code_blocks(&result.md_text)
        .into_iter()
        .map(Into::into)
        .collect();

    let scrap_json = ScrapJson {
        title: result.title.to_string(),
        ctx: result.ctx.map(|c| c.to_string()),
        md_text: result.md_text,
        headings: headings_json,
        code_blocks: code_blocks_json,
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
