use crate::ir::loader;
use crate::mcp::json::scrap::ScrapRefJson;
use crate::mcp::scrap_target;
use crate::usecase::scrap::lookup_neighborhood::usecase::{
    LookupScrapNeighborhoodUsecase, NeighborhoodEdge, NeighborhoodNode,
};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer};
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_DEPTH: usize = 1;
// Edges repeat every scrap they join, so ~50 nodes is where a map still reads cheaply.
const DEFAULT_LIMIT: usize = 50;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupScrapNeighborhoodRequest {
    /// Key of the scrap to open the neighborhood around: its path under the wiki root without `.md`, e.g. "tables/orders". Pass the `key` of a result back.
    pub key: Option<String>,
    /// Still accepted in place of `key`: the scrap's file stem (not its frontmatter title).
    pub title: Option<String>,
    /// Still accepted with `title`: the context (folder path) of the scrap.
    pub ctx: Option<String>,
    /// How many hops to walk out from the scrap (default: 1, capped at 5)
    pub depth: Option<usize>,
    /// Maximum number of nodes in the response (default: 50)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct NeighborhoodNodeJson {
    #[serde(flatten)]
    pub scrap: ScrapRefJson,
    pub hop: usize,
}

#[derive(Debug, Serialize)]
pub struct NeighborhoodEdgeJson {
    pub from: ScrapRefJson,
    pub to: ScrapRefJson,
}

#[derive(Debug, Serialize)]
pub struct LookupScrapNeighborhoodResponse {
    pub nodes: Vec<NeighborhoodNodeJson>,
    pub edges: Vec<NeighborhoodEdgeJson>,
    pub count: usize,
    pub truncated: bool,
    pub dropped: usize,
    pub next: String,
}

impl From<&NeighborhoodNode> for NeighborhoodNodeJson {
    fn from(node: &NeighborhoodNode) -> Self {
        NeighborhoodNodeJson {
            scrap: (&node.scrap).into(),
            hop: node.hop,
        }
    }
}

impl From<&NeighborhoodEdge> for NeighborhoodEdgeJson {
    fn from(edge: &NeighborhoodEdge) -> Self {
        NeighborhoodEdgeJson {
            from: (&edge.from).into(),
            to: (&edge.to).into(),
        }
    }
}

pub async fn lookup_scrap_neighborhood(
    scraps_dir: &Path,
    exclude_dirs: &[std::path::PathBuf],
    _context: RequestContext<RoleServer>,
    Parameters(request): Parameters<LookupScrapNeighborhoodRequest>,
) -> Result<CallToolResult, ErrorData> {
    let scraps = loader::load_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let key = scrap_target::resolve(
        request.key.as_deref(),
        request.title.as_deref(),
        request.ctx.as_deref(),
    )?;

    let result = LookupScrapNeighborhoodUsecase::new()
        .execute(
            &scraps,
            &key,
            request.depth.unwrap_or(DEFAULT_DEPTH),
            request.limit.unwrap_or(DEFAULT_LIMIT),
        )
        .map_err(|e| {
            ErrorData::new(
                ErrorCode(-32004),
                format!("Lookup scrap neighborhood failed: {e}"),
                None,
            )
        })?;

    let count = result.nodes.len();
    let truncated = result.dropped > 0;
    let next = if result.edges.is_empty() {
        "No neighbors: nothing links to or from this scrap. Look for related wording with search_scraps, or open the topic map with list_tags."
    } else if truncated {
        "Read a node by passing its key back: get_scrap {key}; the node cap cut the map short, so raise limit or lower depth for the rest."
    } else {
        "Read a node by passing its key back: get_scrap {key}; widen the map with depth (up to 5 hops)."
    };

    let response = LookupScrapNeighborhoodResponse {
        nodes: result.nodes.iter().map(Into::into).collect(),
        edges: result.edges.iter().map(Into::into).collect(),
        count,
        truncated,
        dropped: result.dropped,
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
