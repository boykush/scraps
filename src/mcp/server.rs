use std::path::PathBuf;

use super::tools::list_tags::list_tags;
use super::tools::search_scraps::{search_scraps, SearchRequest};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer};
use scraps_libs::model::base_url::BaseUrl;

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

#[tool_router]
impl ScrapsServer {
    #[tool(description = "Search scraps")]
    async fn search_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        search_scraps(&self.scraps_dir, &self.base_url, context, parameters).await
    }

    #[tool(description = "List tags")]
    async fn list_tags(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        list_tags(&self.scraps_dir, context).await
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
