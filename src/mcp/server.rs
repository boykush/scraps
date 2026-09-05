use std::path::PathBuf;

use super::tools::get_scrap::{get_scrap, GetScrapRequest};
use super::tools::list_tags::list_tags;
use super::tools::lookup_scrap_backlinks::{lookup_scrap_backlinks, LookupScrapBacklinksRequest};
use super::tools::lookup_scrap_links::{lookup_scrap_links, LookupScrapLinksRequest};
use super::tools::lookup_scrap_neighborhood::{
    lookup_scrap_neighborhood, LookupScrapNeighborhoodRequest,
};
use super::tools::lookup_tag_backlinks::{lookup_tag_backlinks, LookupTagBacklinksRequest};
use super::tools::orient::orient;
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
        description = "Use when you start a session on an unfamiliar wiki: returns its scale (scrap and tag counts), folder contexts, and top tags in one response, with no arguments. Continue with search_scraps for content, or expand a topic with lookup_tag_backlinks; the full tag list is list_tags."
    )]
    async fn orient(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        orient(&self.scraps_dir, &self.exclude_dirs, context).await
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
        description = "Use when one relation at a time is too slow: returns the neighborhood around a scrap as a graph — the scraps within a few hops, each with its distance, and the wiki links between them in both directions. Bodies stay out; read any node with get_scrap. Widen with depth (up to 5 hops) and keep the response small with limit."
    )]
    async fn lookup_scrap_neighborhood(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapNeighborhoodRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_neighborhood(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
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
                 lookup_scrap_links and lookup_scrap_backlinks. For the shape around a scrap \
                 rather than one relation at a time, lookup_scrap_neighborhood returns its \
                 neighborhood as a graph: nodes with their hop distance and the links between \
                 them, no bodies. For the topic map, list_tags then lookup_tag_backlinks.",
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

        assert_eq!(tools.tools.len(), 8);

        let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"orient"));
        assert!(tool_names.contains(&"get_scrap"));
        assert!(tool_names.contains(&"search_scraps"));
        assert!(tool_names.contains(&"lookup_scrap_links"));
        assert!(tool_names.contains(&"lookup_scrap_neighborhood"));
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

    async fn call_tool_json(
        project: &TempScrapProject,
        name: &str,
        arguments: serde_json::Value,
    ) -> serde_json::Value {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );

        let (client_stream, server_stream) = tokio::io::duplex(4096);

        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });

        let client = ().serve(client_stream).await.unwrap();

        let mut params = CallToolRequestParams::new(name.to_string());
        if let Some(args) = arguments.as_object() {
            if !args.is_empty() {
                params = params.with_arguments(args.clone());
            }
        }
        let result = client.call_tool(params).await.unwrap();

        client.cancel().await.unwrap();
        server_handle.abort();

        serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap()
    }

    // Automates livt://mapping/follow-response-hints/rule/R-01
    #[rstest]
    #[tokio::test]
    async fn test_every_response_carries_a_next_hint(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("source.md", b"# Source\n\n[[target]] #[[rust]]");
        project.add_scrap("target.md", b"# Target\n\nContent");

        let calls = vec![
            ("search_scraps", serde_json::json!({"query": "target"})),
            ("get_scrap", serde_json::json!({"title": "target"})),
            ("lookup_scrap_links", serde_json::json!({"title": "source"})),
            (
                "lookup_scrap_backlinks",
                serde_json::json!({"title": "target"}),
            ),
            ("list_tags", serde_json::json!({})),
            ("lookup_tag_backlinks", serde_json::json!({"tag": "rust"})),
            ("orient", serde_json::json!({})),
            (
                "lookup_scrap_neighborhood",
                serde_json::json!({"title": "source"}),
            ),
        ];

        for (name, args) in calls {
            let response = call_tool_json(&project, name, args).await;
            let next = response["next"].as_str().unwrap_or_default();
            assert!(
                !next.is_empty(),
                "{name} response should carry a next hint: {response}"
            );
        }

        let tags = call_tool_json(&project, "list_tags", serde_json::json!({})).await;
        assert!(
            tags["results"].is_array() && tags["count"].is_u64(),
            "list_tags should use the results/count envelope: {tags}"
        );
    }

    // Automates livt://mapping/orient-at-session-start/rule/R-01
    #[rstest]
    #[tokio::test]
    async fn test_orient_gathers_scale_contexts_and_top_tags(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"# A\n\n#[[rust]] #[[go]]");
        project.add_scrap("Kubernetes/b.md", b"# B\n\n#[[rust]]");
        project.add_scrap("Book/c.md", b"# C\n\nplain");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        assert_eq!(orient["scrap_count"], 3);
        assert_eq!(orient["tag_count"], 2);
        assert_eq!(
            orient["contexts"],
            serde_json::json!(["Book", "Kubernetes"])
        );
        assert_eq!(orient["top_tags"][0]["title"], "rust");
    }

    // Automates livt://mapping/orient-at-session-start/rule/R-02
    #[rstest]
    #[tokio::test]
    async fn test_orient_ranks_and_caps_top_tags(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let all_tags: String = (1..=12).map(|i| format!("#[[t{i:02}]] ")).collect();
        project.add_scrap("a.md", format!("# A\n\n{all_tags}").as_bytes());
        project.add_scrap("b.md", b"# B\n\n#[[t12]]");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        assert_eq!(orient["tag_count"], 12);
        assert_eq!(orient["top_tags"].as_array().unwrap().len(), 10);
        assert_eq!(orient["top_tags"][0]["title"], "t12");
    }

    // Automates livt://mapping/orient-at-session-start/rule/R-03
    #[rstest]
    #[tokio::test]
    async fn test_orient_names_entry_tools_in_next(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"# A\n\nContent");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        let next = orient["next"].as_str().unwrap_or_default();
        assert!(
            next.contains("search_scraps") && next.contains("lookup_tag_backlinks"),
            "orient next should name the entry tools: {orient}"
        );
    }

    // Automates livt://mapping/follow-response-hints/rule/R-02
    #[rstest]
    #[tokio::test]
    async fn test_empty_responses_teach_another_way_in(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("target.md", b"# Target\n\nContent");

        let search = call_tool_json(
            &project,
            "search_scraps",
            serde_json::json!({"query": "zzzqqqxxx"}),
        )
        .await;
        assert_eq!(search["count"], 0);
        assert!(
            search["next"]
                .as_str()
                .unwrap_or_default()
                .contains("list_tags"),
            "empty search should point at the topic map: {search}"
        );

        let links = call_tool_json(
            &project,
            "lookup_scrap_links",
            serde_json::json!({"title": "target"}),
        )
        .await;
        assert_eq!(links["count"], 0);
        assert!(
            links["next"]
                .as_str()
                .unwrap_or_default()
                .contains("lookup_scrap_backlinks"),
            "empty links should point at the inbound direction: {links}"
        );
    }

    // Automates livt://mapping/follow-response-hints/rule/R-03
    #[rstest]
    #[tokio::test]
    async fn test_next_stays_top_level_only(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("target.md", b"# Target\n\nContent");

        let search = call_tool_json(
            &project,
            "search_scraps",
            serde_json::json!({"query": "target"}),
        )
        .await;
        assert!(search["count"].as_u64().unwrap() > 0);

        let first = search["results"][0].as_object().unwrap();
        assert!(
            !first.contains_key("next"),
            "result elements should stay lean: {search}"
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

    // Automates livt://mapping/recall-in-one-call/rule/R-01
    #[rstest]
    #[tokio::test]
    async fn test_neighborhood_returns_the_map_in_one_call(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("microservices.md", b"# microservices\n\n[[ddd]]");
        project.add_scrap("ddd.md", b"# ddd\n\nContent");
        project.add_scrap("monolith.md", b"# monolith\n\n[[microservices]]");

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"title": "microservices"}),
        )
        .await;

        assert_eq!(map["count"], 3);
        assert_eq!(map["nodes"][0]["title"], "microservices");
        assert_eq!(map["nodes"][0]["hop"], 0);
        assert_eq!(map["edges"].as_array().unwrap().len(), 2);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-02
    #[rstest]
    #[tokio::test]
    async fn test_neighborhood_leaves_bodies_to_get_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "root.md",
            b"# root\n\nbodytextthatshouldnotberepeated [[other]]",
        );
        project.add_scrap("other.md", b"# other\n\nContent");

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"title": "root"}),
        )
        .await;

        assert!(
            !map.to_string().contains("bodytextthatshouldnotberepeated"),
            "the map should carry no bodies: {map}"
        );
        let node = map["nodes"][0].as_object().unwrap();
        assert_eq!(
            node.keys().cloned().collect::<Vec<_>>(),
            vec!["ctx", "hop", "title"],
            "a node carries only its key and distance"
        );
        assert!(
            map["next"]
                .as_str()
                .unwrap_or_default()
                .contains("get_scrap"),
            "the map should send reading to get_scrap: {map}"
        );
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-01
    #[rstest]
    #[tokio::test]
    async fn test_a_scrap_with_no_relations_teaches_another_way_in(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("lonely.md", b"# lonely\n\nContent");

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"title": "lonely"}),
        )
        .await;

        assert_eq!(map["count"], 1);
        assert!(map["edges"].as_array().unwrap().is_empty());
        assert!(
            map["next"]
                .as_str()
                .unwrap_or_default()
                .contains("list_tags"),
            "a map with no edges should point at another way in: {map}"
        );
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-05
    #[rstest]
    #[tokio::test]
    async fn test_neighborhood_opens_one_hop_when_depth_is_left_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("s0.md", b"# s0\n\n[[s1]]");
        project.add_scrap("s1.md", b"# s1\n\n[[s2]]");
        project.add_scrap("s2.md", b"# s2\n\nContent");

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"title": "s0"}),
        )
        .await;

        assert_eq!(map["count"], 2);
        assert_eq!(map["nodes"][1]["hop"], 1);
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-06
    #[rstest]
    #[tokio::test]
    async fn test_neighborhood_says_when_the_cap_cut_the_map(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let hub_links: String = (1..=20).map(|i| format!("[[n{i:02}]] ")).collect();
        project.add_scrap("hub.md", format!("# hub\n\n{hub_links}").as_bytes());
        for i in 1..=20 {
            project.add_scrap(&format!("n{i:02}.md"), b"Content");
        }

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"title": "hub", "depth": 2, "limit": 5}),
        )
        .await;

        assert_eq!(map["count"], 5);
        assert_eq!(map["truncated"], true);
        assert_eq!(map["dropped"], 16);
        assert!(
            map["next"].as_str().unwrap_or_default().contains("limit"),
            "a cut map should say how to see the rest: {map}"
        );
    }

    // Automates livt://mapping/recall-in-one-call/rule/R-07
    #[rstest]
    fn test_instructions_teach_the_neighborhood_map(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let instructions = server.get_info().instructions.unwrap();

        assert!(
            instructions.contains("lookup_scrap_neighborhood"),
            "the flow should name the neighborhood map: {instructions}"
        );
    }
}
