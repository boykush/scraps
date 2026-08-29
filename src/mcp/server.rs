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
    exclude_dirs: Vec<PathBuf>,
}

impl ScrapsServer {
    pub fn new(scraps_dir: PathBuf, exclude_dirs: Vec<PathBuf>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            scraps_dir,
            exclude_dirs,
        }
    }
}

#[tool_router]
impl ScrapsServer {
    #[tool(
        description = "Use when you know which scrap to read, by title and optional context. Optionally restrict to a heading section via 'heading', and project specific fields via 'fields' (allowed: title, ctx, body, headings, code_blocks; defaults to ['title', 'ctx', 'body']) to keep the response small. Traverse onward with lookup_scrap_links or lookup_scrap_backlinks."
    )]
    async fn get_scrap(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<GetScrapRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        get_scrap(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you have keywords or a rough cue but no exact title. Fuzzy-matches titles and body content; space-separated keywords use OR logic by default (any keyword matches). Start broad, then re-search with logic 'and' to require every keyword. Read promising hits with get_scrap."
    )]
    async fn search_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        search_scraps(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you want what a scrap references: returns its outbound wiki links as scraps. Read any of them with get_scrap."
    )]
    async fn lookup_scrap_links(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapLinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_links(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you want what references a scrap: returns the scraps linking to it (inbound wiki links). Read any of them with get_scrap."
    )]
    async fn lookup_scrap_backlinks(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapBacklinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_backlinks(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you need the wiki's topic map before drilling in: lists all tags used across scraps. Expand a tag into its scraps with lookup_tag_backlinks."
    )]
    async fn list_tags(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        list_tags(&self.scraps_dir, &self.exclude_dirs, context).await
    }

    #[tool(
        description = "Use when you want everything filed under a topic: returns the scraps referencing a tag. Read individual results with get_scrap."
    )]
    async fn lookup_tag_backlinks(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupTagBacklinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_tag_backlinks(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }
}

// Without `router = ...` the macro rebuilds the router (and every tool's
// JSON schema) on each request; point it at the one built in `new`.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for ScrapsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Read interface to a Scraps wiki: markdown scraps typed with wiki-links, tags, \
                 and folder contexts. Recommended flow: start with search_scraps on broad OR \
                 keywords and narrow with logic 'and'; read the best hits with get_scrap, \
                 projecting fields to keep responses small; then traverse relations with \
                 lookup_scrap_links and lookup_scrap_backlinks. For the topic map, list_tags \
                 then lookup_tag_backlinks.",
        )
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
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let info = server.get_info();

        assert!(info.instructions.is_some());
        assert!(info.capabilities.tools.is_some());
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_tools(#[from(temp_scrap_project)] project: TempScrapProject) {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

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

    async fn list_tools_of(project: &TempScrapProject) -> Vec<rmcp::model::Tool> {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let tools = client.list_tools(Default::default()).await.unwrap().tools;

        client.cancel().await.unwrap();
        server_handle.abort();
        tools
    }

    // Automates livt://mapping/learn-usage-from-tool-defs/rule/R-01
    #[rstest]
    fn test_instructions_teach_the_drawing_flow(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let instructions = server.get_info().instructions.unwrap();

        assert_ne!(instructions, "This is a Scraps MCP server");

        let search = instructions.find("search_scraps").unwrap();
        let get = instructions.find("get_scrap").unwrap();
        let links = instructions.find("lookup_scrap_links").unwrap();
        assert!(
            search < get && get < links,
            "flow should read search -> get -> links: {instructions}"
        );
    }

    // Automates livt://mapping/learn-usage-from-tool-defs/rule/R-02
    #[rstest]
    #[tokio::test]
    async fn test_tool_descriptions_open_with_when_to_use(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        for tool in list_tools_of(&project).await {
            let desc = tool.description.as_deref().unwrap_or_default();
            assert!(
                desc.starts_with("Use when"),
                "{} should open with when to use it: {desc}",
                tool.name
            );
        }
    }

    // Automates livt://mapping/learn-usage-from-tool-defs/rule/R-03
    #[rstest]
    #[tokio::test]
    async fn test_tool_descriptions_name_a_follow_up_tool(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tools = list_tools_of(&project).await;
        let names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();

        for tool in &tools {
            let desc = tool.description.as_deref().unwrap_or_default();
            let names_another = names
                .iter()
                .any(|name| name != tool.name.as_ref() && desc.contains(name.as_str()));
            assert!(
                names_another,
                "{} should point at a follow-up tool: {desc}",
                tool.name
            );
        }
    }

    // Automates livt://mapping/learn-usage-from-tool-defs/rule/R-04
    #[rstest]
    #[tokio::test]
    async fn test_tool_descriptions_carry_argument_etiquette(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tools = list_tools_of(&project).await;
        let desc_of = |name: &str| {
            tools
                .iter()
                .find(|t| t.name.as_ref() == name)
                .unwrap()
                .description
                .as_deref()
                .unwrap_or_default()
                .to_string()
        };

        let search = desc_of("search_scraps");
        assert!(
            search.contains("broad") && search.contains("logic 'and'"),
            "search should teach broaden-then-narrow: {search}"
        );

        let get = desc_of("get_scrap");
        assert!(
            get.contains("fields") && get.contains("small"),
            "get should teach field-projection thrift: {get}"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_search_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test Scrap\n\nContent here");

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(
                CallToolRequestParams::new("search_scraps").with_arguments(
                    serde_json::json!({"query": "test"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
            .await
            .unwrap();

        assert!(!result.is_error.unwrap_or(false));
        assert!(!result.content.is_empty());

        let content_text = result.content[0].as_text().unwrap();
        let response: serde_json::Value = serde_json::from_str(&content_text.text).unwrap();
        assert!(response["count"].as_u64().unwrap() > 0);
        assert!(content_text.text.contains("test"));
        assert!(
            !content_text.text.contains("md_text"),
            "search_scraps should not include md_text"
        );

        client.cancel().await.unwrap();
        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_get_scrap(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test Scrap\n\nContent here");

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(
                CallToolRequestParams::new("get_scrap").with_arguments(
                    serde_json::json!({"title": "test"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(CallToolRequestParams::new("list_tags"))
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(
                CallToolRequestParams::new("lookup_scrap_links").with_arguments(
                    serde_json::json!({"title": "source"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(
                CallToolRequestParams::new("lookup_scrap_backlinks").with_arguments(
                    serde_json::json!({"title": "target"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let result = client
            .call_tool(
                CallToolRequestParams::new("lookup_tag_backlinks").with_arguments(
                    serde_json::json!({"tag": "rust"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        // AND search: "rust python" should only match "rust_python.md"
        let result = client
            .call_tool(
                CallToolRequestParams::new("search_scraps").with_arguments(
                    serde_json::json!({"query": "rust python", "logic": "and"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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
            content_text.text.contains("rust_python"),
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

        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        // OR search: "rust python" should match all 3 scraps
        let result = client
            .call_tool(
                CallToolRequestParams::new("search_scraps").with_arguments(
                    serde_json::json!({"query": "rust python", "logic": "or"})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )
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
