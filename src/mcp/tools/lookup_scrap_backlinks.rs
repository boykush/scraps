use crate::ir::loader;
use crate::mcp::json::scrap::ScrapRefJson;
use crate::mcp::scrap_target;
use crate::usecase::scrap::lookup_backlinks::usecase::LookupScrapBacklinksUsecase;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupScrapBacklinksRequest {
    /// Key of the scrap to get backlinks for: its path under the wiki root without `.md`, e.g. "tables/orders". Pass the `key` of a result back.
    pub key: Option<String>,
    /// Still accepted in place of `key`: the scrap's file stem (not its frontmatter title).
    pub title: Option<String>,
    /// Still accepted with `title`: the context (folder path) of the scrap.
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapBacklinksResponse {
    pub results: Vec<ScrapRefJson>,
    pub count: usize,
    pub next: String,
}

pub async fn lookup_scrap_backlinks(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapBacklinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    // Create get scrap backlinks usecase
    let get_backlinks_usecase = LookupScrapBacklinksUsecase::new();

    // Execute get backlinks
    let key = scrap_target::resolve(
        request.key.as_deref(),
        request.title.as_deref(),
        request.ctx.as_deref(),
    )?;

    let results = get_backlinks_usecase.execute(&scraps, &key).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32006),
            format!("Get scrap backlinks failed: {e}"),
            None,
        )
    })?;

    // Convert results to structured response
    let scrap_jsons: Vec<ScrapRefJson> = results
        .iter()
        .map(|result| (&result.scrap).into())
        .collect();

    let count = scrap_jsons.len();
    let next = if count == 0 {
        "No backlinks. Try lookup_scrap_links {key} for outbound references."
    } else {
        "Read a linking scrap by passing its key back: get_scrap {key}."
    };
    let response = LookupScrapBacklinksResponse {
        results: scrap_jsons,
        count,
        next: next.to_string(),
    };

    Ok(CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string(&response).map_err(|e| {
            ErrorData::new(
                ErrorCode(-32007),
                format!("JSON serialization failed: {e}"),
                None,
            )
        })?,
    )]))
}
