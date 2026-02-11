use std::path::PathBuf;

use super::tools::get_scrap::{get_scrap, GetScrapRequest};
use super::tools::list_tags::list_tags;
use super::tools::lookup_scrap_backlinks::{lookup_scrap_backlinks, LookupScrapBacklinksRequest};
use super::tools::lookup_scrap_links::{lookup_scrap_links, LookupScrapLinksRequest};
use super::tools::lookup_tag_backlinks::{lookup_tag_backlinks, LookupTagBacklinksRequest};
use super::tools::search_scraps::{search_scraps, SearchRequest};
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::service::RequestContext;
use rmcp::{tool, tool_handler, tool_router, ErrorData, RoleServer};

pub struct ScrapsServer {
    tool_router: ToolRouter<ScrapsServer>,
    scraps_dir: PathBuf,
}

impl ScrapsServer {
    pub fn new(scraps_dir: PathBuf) -> Self {
        Self {
            tool_router: Self::tool_router(),
            scraps_dir,
        }
    }
}

#[tool_router]
impl ScrapsServer {
    #[tool(
        description = "Get a single scrap by title and optional context. Returns the scrap's full content including title, context, and markdown body."
    )]
    async fn get_scrap(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<GetScrapRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        get_scrap(&self.scraps_dir, context, parameters).await
    }

    #[tool(
        description = "Search for scraps using fuzzy matching against title and body content. Space-separated keywords use OR logic by default (any keyword matches). Set logic to 'and' for all keywords to match. Returns matching scraps with titles, contexts, and full content."
    )]
    async fn search_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        search_scraps(&self.scraps_dir, context, parameters).await
    }

    #[tool(
        description = "Lookup outbound wiki links from a specific scrap. Returns all scraps that the specified scrap links to, with their full content."
    )]
    async fn lookup_scrap_links(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapLinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_links(&self.scraps_dir, context, parameters).await
    }

    #[tool(
        description = "Lookup inbound wiki links (backlinks) to a specific scrap. Returns all scraps that link to the specified scrap, with their full content."
    )]
    async fn lookup_scrap_backlinks(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapBacklinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_backlinks(&self.scraps_dir, context, parameters).await
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

    #[tool(
        description = "Lookup inbound references (backlinks) to a specific tag. Returns all scraps that reference the specified tag, with their full content."
    )]
    async fn lookup_tag_backlinks(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupTagBacklinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_tag_backlinks(&self.scraps_dir, context, parameters).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use rmcp::model::CallToolRequestParams;
    use rmcp::ServiceExt;
    use rstest::rstest;

    #[rstest]
    fn test_server_info(#[from(temp_scrap_project)] project: TempScrapProject) {
        let server = ScrapsServer::new(project.scraps_dir.clone());
        let info = server.get_info();

        assert_eq!(
            info.instructions,
            Some("This is a Scraps MCP server".into())
        );
        assert!(info.capabilities.tools.is_some());
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_tools(#[from(temp_scrap_project)] project: TempScrapProject) {
        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let tools = client.list_tools(Default::default()).await.unwrap();

        assert_eq!(tools.tools.len(), 6);

        let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"get_scrap"));
        assert!(tool_names.contains(&"search_scraps"));
        assert!(tool_names.contains(&"lookup_scrap_links"));
        assert!(tool_names.contains(&"lookup_scrap_backlinks"));
        assert!(tool_names.contains(&"list_tags"));
        assert!(tool_names.contains(&"lookup_tag_backlinks"));

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_search_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test Scrap\n\nContent here");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_scraps".into(),
                arguments: Some(
                    serde_json::json!({"query": "test"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(content_text.text.contains("Test Scrap"));

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_get_scrap(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test Scrap\n\nContent here");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "get_scrap".into(),
                arguments: Some(
                    serde_json::json!({"title": "test"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(content_text.text.contains("Test Scrap"));
        assert!(content_text.text.contains("Content here"));

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_list_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"#[[rust]] #[[programming]]");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "list_tags".into(),
                arguments: None,
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(
            content_text.text.contains("rust") || content_text.text.contains("programming"),
            "Expected tags in response, got: {}",
            content_text.text
        );

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_lookup_scrap_links(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("source.md", b"# Source\n\n[[target]]");
        project.add_scrap("target.md", b"# Target\n\nTarget content");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "lookup_scrap_links".into(),
                arguments: Some(
                    serde_json::json!({"title": "source"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(content_text.text.contains("target"));

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_lookup_scrap_backlinks(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("source.md", b"# Source\n\n[[target]]");
        project.add_scrap("target.md", b"# Target\n\nTarget content");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "lookup_scrap_backlinks".into(),
                arguments: Some(
                    serde_json::json!({"title": "target"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(content_text.text.contains("source"));

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_lookup_tag_backlinks(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test\n\n#[[rust]]");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "lookup_tag_backlinks".into(),
                arguments: Some(
                    serde_json::json!({"tag": "rust"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        assert!(
            content_text.text.contains("Test") || content_text.text.contains("test"),
            "Expected scrap with tag 'rust' in response, got: {}",
            content_text.text
        );

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    /// Test: search_scraps with AND logic (default) - all keywords must match
    #[rstest]
    #[tokio::test]
    async fn test_call_search_scraps_and_logic(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        // Setup: 3 scraps - only one contains both "rust" and "python"
        project.add_scrap("rust_doc.md", b"# Rust Documentation\n\nRust content");
        project.add_scrap("python_doc.md", b"# Python Documentation\n\nPython content");
        project.add_scrap("rust_python.md", b"# Rust and Python\n\nBoth languages");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        // AND search: "rust python" should only match "rust_python.md"
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_scraps".into(),
                arguments: Some(
                    serde_json::json!({"query": "rust python", "logic": "and"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        // Parse the JSON response to check count
        let response: serde_json::Value = serde_json::from_str(&content_text.text).unwrap();
        assert_eq!(
            response["count"], 1,
            "AND search should return only 1 result matching both keywords"
        );
        assert!(
            content_text.text.contains("Rust and Python"),
            "AND search should match the scrap containing both keywords"
        );

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    /// Test: search_scraps with OR logic - any keyword can match
    #[rstest]
    #[tokio::test]
    async fn test_call_search_scraps_or_logic(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        // Setup: 3 scraps - all contain either "rust" or "python"
        project.add_scrap("rust_doc.md", b"# Rust Documentation\n\nRust content");
        project.add_scrap("python_doc.md", b"# Python Documentation\n\nPython content");
        project.add_scrap("rust_python.md", b"# Rust and Python\n\nBoth languages");

        let server = ScrapsServer::new(project.scraps_dir.clone());

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        // OR search: "rust python" should match all 3 scraps
        let result = client
            .call_tool(CallToolRequestParams {
                meta: None,
                name: "search_scraps".into(),
                arguments: Some(
                    serde_json::json!({"query": "rust python", "logic": "or"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
                task: None,
            })
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        // Parse the JSON response to check count
        let response: serde_json::Value = serde_json::from_str(&content_text.text).unwrap();
        assert_eq!(
            response["count"], 3,
            "OR search should return all 3 results matching any keyword"
        );

        client.cancel().await.unwrap();
        server_handle.abort();
    }
}
