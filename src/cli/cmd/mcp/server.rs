use std::future::Future;

use rmcp::handler::server::tool::{Parameters, ToolRouter};
use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer};
use serde::Deserialize;
use serde_json::{json, Map};

pub struct ScrapsServer {
    tool_router: ToolRouter<ScrapsServer>,
}

impl ScrapsServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for ScrapsServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
struct HelloWorldRequest {
    // Empty struct with proper object schema
}

#[tool_router]
impl ScrapsServer {
    #[tool(description = "Hello World")]
    async fn hello_world(
        &self,
        _context: RequestContext<RoleServer>,
        Parameters(_request): Parameters<HelloWorldRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut map = Map::new();
        map.insert("message".into(), json!("Hello, World!"));
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{}",
            json!(map)
        ))]))
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
