use std::path::PathBuf;

use super::tools::list_tags::list_tags;
use super::tools::lookup_scrap_backlinks::{lookup_scrap_backlinks, LookupScrapBacklinksRequest};
use super::tools::lookup_scrap_links::{lookup_scrap_links, LookupScrapLinksRequest};
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
    #[tool(
        description = "Search for scraps by title and context (ctx) using fuzzy matching. Returns matching scraps with their titles, contexts, and full content."
    )]
    async fn search_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        search_scraps(&self.scraps_dir, &self.base_url, context, parameters).await
    }

    #[tool(
        description = "Lookup outbound wiki links from a specific scrap. Returns all scraps that the specified scrap links to, with their full content."
    )]
    async fn lookup_scrap_links(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapLinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_links(&self.scraps_dir, &self.base_url, context, parameters).await
    }

    #[tool(
        description = "Lookup inbound wiki links (backlinks) to a specific scrap. Returns all scraps that link to the specified scrap, with their full content."
    )]
    async fn lookup_scrap_backlinks(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapBacklinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_backlinks(&self.scraps_dir, &self.base_url, context, parameters).await
    }

    #[tool(
        description = "List all available tags used across scraps in the documentation site. Useful for discovering content categories and topics."
    )]
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
