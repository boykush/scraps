use crate::input::file::read_scraps;
use crate::mcp::json::scrap::ScrapKeyJson;
use crate::usecase::scrap::lookup_neighborhood::usecase::{
    LookupScrapNeighborhoodUsecase, NeighborhoodEdge, NeighborhoodNode, ScrapRef,
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
// Edges repeat titles, so ~50 nodes is where a map still reads cheaply.
const DEFAULT_LIMIT: usize = 50;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct LookupScrapNeighborhoodRequest {
    /// Title of the scrap to open the neighborhood around
    pub title: String,
    /// Optional context if the scrap has one
    pub ctx: Option<String>,
    /// How many hops to walk out from the scrap (default: 1, capped at 5)
    pub depth: Option<usize>,
    /// Maximum number of nodes in the response (default: 50)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct NeighborhoodNodeJson {
    pub title: String,
    pub ctx: Option<String>,
    pub hop: usize,
}

#[derive(Debug, Serialize)]
pub struct NeighborhoodEdgeJson {
    pub from: ScrapKeyJson,
    pub to: ScrapKeyJson,
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

impl From<&ScrapRef> for ScrapKeyJson {
    fn from(scrap: &ScrapRef) -> Self {
        ScrapKeyJson {
            title: scrap.title.to_string(),
            ctx: scrap.ctx.as_ref().map(|c| c.to_string()),
        }
    }
}

impl From<&NeighborhoodNode> for NeighborhoodNodeJson {
    fn from(node: &NeighborhoodNode) -> Self {
        NeighborhoodNodeJson {
            title: node.scrap.title.to_string(),
            ctx: node.scrap.ctx.as_ref().map(|c| c.to_string()),
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
    let scraps = read_scraps::to_all_scraps(scraps_dir, exclude_dirs).map_err(|e| {
        ErrorData::new(
            ErrorCode(-32003),
            format!("Failed to load scraps: {e}"),
            None,
        )
    })?;

    let title = scraps_libs::model::title::Title::from(request.title.as_str());
    let ctx = request
        .ctx
        .as_ref()
        .map(|c| scraps_libs::model::context::Ctx::from(c.as_str()));

    let result = LookupScrapNeighborhoodUsecase::new()
        .execute(
            &scraps,
            &title,
            &ctx,
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
        "Read a node with get_scrap {title, ctx}; the node cap cut the map short, so raise limit or lower depth for the rest."
    } else {
        "Read a node with get_scrap {title, ctx}; widen the map with depth (up to 5 hops)."
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
