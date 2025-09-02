use std::path::PathBuf;

use crate::usecase::search::usecase::SearchUsecase;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::ErrorCode;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer};
use scraps_libs::model::base_url::BaseUrl;
use serde::Deserialize;
use serde_json::json;

pub struct ScrapsServer {
    tool_router: ToolRouter<ScrapsServer>,
    scraps_dir: PathBuf,
    base_url: BaseUrl,
}

impl ScrapsServer {
    pub fn new(scraps_dir: PathBuf, base_url: BaseUrl) -> Self {
        Self {
            tool_router: Self::tool_router(),
            scraps_dir,
            base_url,
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
struct SearchRequest {
    /// Query string to search for
    query: String,
    /// Maximum number of results to return (default: 100)
    num: Option<usize>,
}

#[tool_router]
impl ScrapsServer {
    #[tool(description = "Search scraps")]
    async fn search(
        &self,
        _context: RequestContext<RoleServer>,
        Parameters(request): Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        // Create search usecase
        let search_usecase = SearchUsecase::new(&self.scraps_dir);

        // Execute search
        let num = request.num.unwrap_or(100);
        let results = search_usecase
            .execute(&self.base_url, &request.query, num)
            .map_err(|e| ErrorData::new(ErrorCode(-32004), format!("Search failed: {e}"), None))?;

        // Convert results to JSON
        let results_json = results
            .into_iter()
            .map(|result| {
                json!({
                    "title": result.title,
                    "url": result.url
                })
            })
            .collect::<Vec<_>>();

        let response = json!({
            "results": results_json,
            "count": results_json.len()
        });

        Ok(CallToolResult::success(vec![Content::text(
            response.to_string(),
        )]))
    }
}

#[tool_handler]
impl ServerHandler for ScrapsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This is a Scraps MCP server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
