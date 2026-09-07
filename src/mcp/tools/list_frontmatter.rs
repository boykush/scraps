use crate::input::file::read_scraps;
use crate::mcp::json::scrap::ScrapKeyJson;
use crate::usecase::frontmatter::usecase::FrontmatterUsecase;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct FrontmatterItemJson {
    pub scrap: ScrapKeyJson,
    pub frontmatter: Value,
}

#[derive(Debug, Serialize)]
pub struct ListFrontmatterResponse {
    pub results: Vec<FrontmatterItemJson>,
    pub count: usize,
    pub next: String,
}

pub async fn list_frontmatter(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let frontmatter_usecase = FrontmatterUsecase::new();

    let results = frontmatter_usecase.execute(&scraps).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32004),
            format!("List frontmatter failed: {e}"),
            None,
        )
    })?;

    let items: Vec<FrontmatterItemJson> = results
        .into_iter()
        .map(|result| FrontmatterItemJson {
            scrap: ScrapKeyJson {
                title: result.title.to_string(),
                ctx: result.ctx.map(|c| c.to_string()),
            },
            frontmatter: result.frontmatter,
        })
        .collect();

    let count = items.len();
    let next = if count == 0 {
        "No scrap carries frontmatter. A scraps wiki types its own metadata as #[[tag]] instead — start from list_tags."
    } else {
        "Keys are the author's own; scraps gives them no meaning. Read the scrap behind an entry with get_scrap {title, ctx}."
    };
    let response = ListFrontmatterResponse {
        results: items,
        count,
        next: next.to_string(),
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32005),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
