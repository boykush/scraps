use crate::input::file::read_scraps;
use crate::usecase::scrap::lookup_links::usecase::{LinkRefKind, LookupScrapLinksUsecase};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupScrapLinksRequest {
    /// Title of the scrap to get links for
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LinkRefKindJson {
    Link,
    Embed,
}

impl From<LinkRefKind> for LinkRefKindJson {
    fn from(k: LinkRefKind) -> Self {
        match k {
            LinkRefKind::Link => LinkRefKindJson::Link,
            LinkRefKind::Embed => LinkRefKindJson::Embed,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LinkRefJson {
    pub kind: LinkRefKindJson,
    pub title: String,
    pub ctx: Option<String>,
    pub heading: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapLinksResponse {
    pub results: Vec<LinkRefJson>,
    pub count: usize,
}

pub async fn lookup_scrap_links(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapLinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    // Create get scrap links usecase
    let get_links_usecase = LookupScrapLinksUsecase::new();

    // Execute get links
    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let results = get_links_usecase
        .execute(&scraps, &title, &ctx)
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32004),
                format!("Get scrap links failed: {e}"),
                None,
            )
        })?;

    // Convert results to structured response
    let refs: Vec<LinkRefJson> = results
        .into_iter()
        .map(|result| LinkRefJson {
            kind: result.kind.into(),
            title: result.title.to_string(),
            ctx: result.ctx.map(|c| c.to_string()),
            heading: result.heading,
        })
        .collect();

    let count = refs.len();
    let response = LookupScrapLinksResponse {
        results: refs,
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
