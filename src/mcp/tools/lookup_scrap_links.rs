use crate::ir::loader;
use crate::mcp::json::scrap::ScrapRefJson;
use crate::mcp::scrap_target;
use crate::usecase::scrap::lookup_links::usecase::{LinkRefKind, LookupScrapLinksUsecase};
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
pub struct LookupScrapLinksRequest {
    /// Key of the scrap to get links for: its path under the wiki root without `.md`, e.g. "tables/orders". Pass the `key` of a result back.
    pub key: Option<String>,
    /// Still accepted in place of `key`: the scrap's file stem (not its frontmatter title).
    pub title: Option<String>,
    /// Still accepted with `title`: the context (folder path) of the scrap.
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
    #[serde(flatten)]
    pub scrap: ScrapRefJson,
    pub heading: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapLinksResponse {
    pub results: Vec<LinkRefJson>,
    pub count: usize,
    pub next: String,
}

pub async fn lookup_scrap_links(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapLinksRequest>,
) -> Result<CallToolResult, ErrorData> {
    // Load scraps from directory
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    // Create get scrap links usecase
    let get_links_usecase = LookupScrapLinksUsecase::new();

    // Execute get links
    let key = scrap_target::resolve(
        request.key.as_deref(),
        request.title.as_deref(),
        request.ctx.as_deref(),
    )?;

    let results = get_links_usecase.execute(&scraps, &key).map_err(|e| {
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
            scrap: (&result.scrap).into(),
            heading: result.heading,
        })
        .collect();

    let count = refs.len();
    let next = if count == 0 {
        "No outbound links. Try lookup_scrap_backlinks {key} for inbound references."
    } else {
        "Read a linked scrap by passing its key back: get_scrap {key} — add heading to jump to the referenced section."
    };
    let response = LookupScrapLinksResponse {
        results: refs,
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
