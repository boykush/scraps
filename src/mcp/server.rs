use std::path::PathBuf;

use super::tools::get_scrap::{GetScrapRequest, get_scrap};
use super::tools::list_tags::list_tags;
use super::tools::list_todos::{ListTodosRequest, list_todos};
use super::tools::list_types::list_types;
use super::tools::lookup_scrap_backlinks::{LookupScrapBacklinksRequest, lookup_scrap_backlinks};
use super::tools::lookup_scrap_links::{LookupScrapLinksRequest, lookup_scrap_links};
use super::tools::lookup_scrap_neighborhood::{
    LookupScrapNeighborhoodRequest, lookup_scrap_neighborhood,
};
use super::tools::lookup_tag_backlinks::{LookupTagBacklinksRequest, lookup_tag_backlinks};
use super::tools::lookup_type_scraps::{LookupTypeScrapsRequest, lookup_type_scraps};
use super::tools::orient::orient;
use super::tools::search_scraps::{SearchRequest, search_scraps};
use rmcp::handler::server::ServerHandler;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Implementation, ServerCapabilities, ServerConfig};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, tool, tool_handler, tool_router};

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
        description = "Use when you know which scrap to read: pass the `key` of a result back (the scrap's path under the wiki root without `.md`, e.g. 'tables/orders'). The older `title` (file stem) with optional `ctx` is still accepted. Optionally restrict to a heading section via 'heading', and project specific fields via 'fields' (allowed: key, title, ctx, body, headings, code_blocks, type, description, verified, sources, resource; defaults to ['key', 'title', 'ctx', 'body']) to keep the response small. 'body' is the scrap's content without its YAML frontmatter block. The Open Knowledge Format keys Scraps reads from that block are fields of their own: 'type' and 'description', 'verified' for the trust tier and confirmations, 'sources' for the materials the scrap derives from, and 'resource' for the URI of what the scrap is about (null when there is none). Line numbers in headings and code_blocks count lines of the source file, frontmatter block included, so they are not offsets into body. Traverse onward with lookup_scrap_links or lookup_scrap_backlinks, or see everything around the scrap in one call with lookup_scrap_neighborhood."
    )]
    async fn get_scrap(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<GetScrapRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        get_scrap(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you start a session on an unfamiliar wiki: returns its scale (scrap, tag and type counts), folder contexts, top tags, and top types in one response, with no arguments. Continue with search_scraps for content, expand a topic with lookup_tag_backlinks, or expand a kind of scrap with lookup_type_scraps; the full lists are list_tags and list_types."
    )]
    async fn orient(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        orient(&self.scraps_dir, &self.exclude_dirs, context).await
    }

    #[tool(
        description = "Use when you have keywords or a rough cue but no exact key. Fuzzy-matches keys, titles, descriptions and tags, not bodies; space-separated keywords use OR logic by default (any keyword matches). Start broad, then re-search with logic 'and' to require every keyword. Read promising hits by passing their `key` to get_scrap."
    )]
    async fn search_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        search_scraps(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you want what a scrap references: returns its outbound wiki links as scraps, each with its kind: `link`, `embed`, or `source` for a frontmatter `sources` path. Name the scrap by `key`; read any result by passing its `key` to get_scrap."
    )]
    async fn lookup_scrap_links(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapLinksRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_links(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when one relation at a time is too slow: returns the neighborhood around a scrap as a graph — the scraps within a few hops, each with its distance, and the wiki links between them in both directions. Bodies stay out; name the scrap by `key` and read any node by passing its `key` to get_scrap. Widen with depth (up to 5 hops) and keep the response small with limit."
    )]
    async fn lookup_scrap_neighborhood(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupScrapNeighborhoodRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_scrap_neighborhood(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you want what references a scrap: returns the scraps linking to it (inbound wiki links), each with the `kinds` it links by: `link` from its body, `source` from its `sources`. Name the scrap by `key`; read any result by passing its `key` to get_scrap."
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

    #[tool(
        description = "Use when you need what kinds of scraps the wiki holds: lists each type in use with the number of scraps of that type. A type says what a scrap is (at most one, from its frontmatter `type`), where a tag says what it is about; a scrap need not have one. Expand a type into its scraps with lookup_type_scraps."
    )]
    async fn list_types(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        list_types(&self.scraps_dir, &self.exclude_dirs, context).await
    }

    #[tool(
        description = "Use when you want every scrap of one kind, such as all blog posts or all book summaries: returns the scraps whose type is exactly the given one, as list_types spells it. Read individual results with get_scrap."
    )]
    async fn lookup_type_scraps(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<LookupTypeScrapsRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        lookup_type_scraps(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }

    #[tool(
        description = "Use when you want the work recorded across the wiki rather than one scrap's: returns the markdown task list items across every scrap, each with the scrap it sits in and its line. Filter with status — 'open' (default), 'done', 'deferred' or 'all'. Read the scrap around a task with get_scrap."
    )]
    async fn list_todos(
        &self,
        context: RequestContext<RoleServer>,
        parameters: Parameters<ListTodosRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        list_todos(&self.scraps_dir, &self.exclude_dirs, context, parameters).await
    }
}

// Without `router = ...` the macro rebuilds the router (and every tool's
// JSON schema) on each request; point it at the one built in `new`.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for ScrapsServer {
    fn get_info(&self) -> ServerConfig {
        // rmcp's default identity expands `env!` inside rmcp itself, so it would
        // report the SDK's name and version instead of the running scraps.
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Read interface to a Scraps wiki: markdown scraps typed with wiki-links, tags, \
                 and folder contexts. Recommended flow: start with search_scraps on broad OR \
                 keywords and narrow with logic 'and'; read the best hits with get_scrap, \
                 projecting fields to keep responses small; then traverse relations with \
                 lookup_scrap_links and lookup_scrap_backlinks. Every result names a scrap by \
                 its `key` (its path under the wiki root without `.md`, the same string a \
                 wiki-link holds) next to the `title` readers see: pass the `key` back to \
                 reach that scrap. For the shape around a scrap \
                 rather than one relation at a time, lookup_scrap_neighborhood returns its \
                 neighborhood as a graph: nodes with their hop distance and the links between \
                 them, no bodies. For the topic map, list_tags then lookup_tag_backlinks; for \
                 the kinds of scraps, list_types then lookup_type_scraps. For \
                 the work recorded across the wiki, list_todos.",
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use rmcp::ServiceExt;
    use rmcp::model::CallToolRequestParams;
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
    fn test_server_info_identifies_scraps_and_its_version(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let implementation = server.get_info().server_info;

        assert_eq!(implementation.name, "scraps");
        assert_eq!(implementation.version, env!("CARGO_PKG_VERSION"));
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

        assert_eq!(tools.tools.len(), 11);

        let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(tool_names.contains(&"orient"));
        assert!(tool_names.contains(&"get_scrap"));
        assert!(tool_names.contains(&"search_scraps"));
        assert!(tool_names.contains(&"lookup_scrap_links"));
        assert!(tool_names.contains(&"lookup_scrap_neighborhood"));
        assert!(tool_names.contains(&"lookup_scrap_backlinks"));
        assert!(tool_names.contains(&"list_tags"));
        assert!(tool_names.contains(&"lookup_tag_backlinks"));
        assert!(tool_names.contains(&"list_types"));
        assert!(tool_names.contains(&"lookup_type_scraps"));
        assert!(tool_names.contains(&"list_todos"));
        assert!(!tool_names.contains(&"list_frontmatter"));

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
    async fn test_search_schema_names_the_logic_it_defaults_to(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("rust_doc.md", b"# Rust Documentation\n\nRust content");
        project.add_scrap("python_doc.md", b"# Python Documentation\n\nPython content");

        let tools = list_tools_of(&project).await;
        let search = tools
            .iter()
            .find(|t| t.name.as_ref() == "search_scraps")
            .unwrap();
        let schema = serde_json::to_value(&*search.input_schema).unwrap();
        let described = schema["properties"]["logic"]["description"]
            .as_str()
            .unwrap_or_default();

        // Leaving logic out matches either keyword, so that is the default to name.
        let unset = call_tool_json(
            &project,
            "search_scraps",
            serde_json::json!({"query": "rust python"}),
        )
        .await;
        assert_eq!(unset["count"], 2);
        assert!(
            described.contains("\"or\" (default"),
            "the schema should name the default it applies: {described}"
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
        if let Some(args) = arguments.as_object()
            && !args.is_empty()
        {
            params = params.with_arguments(args.clone());
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
            ("list_types", serde_json::json!({})),
            ("lookup_type_scraps", serde_json::json!({"type": "Blog"})),
            ("orient", serde_json::json!({})),
            ("list_todos", serde_json::json!({})),
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

    // Automates livt://mapping/orient-at-session-start/rule/R-04
    #[rstest]
    #[tokio::test]
    async fn test_orient_reports_the_types_in_use(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"---\ntype: Blog\n---\n# A\n\n#[[rust]]");
        project.add_scrap("b.md", b"---\ntype: Book\n---\n# B");
        project.add_scrap("c.md", b"---\ntype: Book\n---\n# C");
        project.add_scrap("d.md", b"---\ntype: Area\n---\n# D");
        project.add_scrap("e.md", b"# E\n\nuntyped");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        assert_eq!(orient["type_count"], 3);
        assert_eq!(
            orient["top_types"],
            serde_json::json!([
                { "type": "Book", "scraps_count": 2 },
                { "type": "Area", "scraps_count": 1 },
                { "type": "Blog", "scraps_count": 1 }
            ])
        );
        let next = orient["next"].as_str().unwrap_or_default();
        assert!(
            next.contains("lookup_tag_backlinks")
                && next.contains("lookup_type_scraps {type}")
                && next.contains("list_types"),
            "orient next should name the type tools beside the tag tools: {orient}"
        );
    }

    // Automates livt://mapping/orient-at-session-start/rule/R-04
    #[rstest]
    #[tokio::test]
    async fn test_orient_caps_top_types(#[from(temp_scrap_project)] project: TempScrapProject) {
        for i in 1..=12 {
            project.add_scrap(
                &format!("s{i:02}.md"),
                format!("---\ntype: T{i:02}\n---\n# S{i:02}").as_bytes(),
            );
        }
        project.add_scrap("extra.md", b"---\ntype: T12\n---\n# Extra");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        assert_eq!(orient["type_count"], 12);
        assert_eq!(orient["top_types"].as_array().unwrap().len(), 10);
        assert_eq!(orient["top_types"][0]["type"], "T12");
    }

    // Automates livt://mapping/orient-at-session-start/rule/R-05
    #[rstest]
    #[tokio::test]
    async fn test_orient_keeps_an_untyped_wiki_free_of_type_hints(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"# A\n\n#[[rust]]");

        let orient = call_tool_json(&project, "orient", serde_json::json!({})).await;

        assert_eq!(orient["type_count"], 0);
        assert_eq!(orient["top_types"], serde_json::json!([]));
        assert_eq!(
            orient["next"],
            "Search content with search_scraps, or expand a top tag with lookup_tag_backlinks {tag}; the full tag list is list_tags."
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
    async fn test_call_list_todos_defaults_to_open(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "Programming/Rust/borrowing.md",
            b"# borrowing\n\n- [ ] implement Drop for X\n- [x] write tests\n- [-] revisit later\n",
        );

        let todos = call_tool_json(&project, "list_todos", serde_json::json!({})).await;

        assert_eq!(todos["count"], 1);
        let item = &todos["results"][0];
        assert_eq!(item["scrap"]["title"], "borrowing");
        assert_eq!(item["scrap"]["ctx"], "Programming/Rust");
        assert_eq!(item["status"], "open");
        assert_eq!(item["text"], "implement Drop for X");
        assert_eq!(item["line"], 3);
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_list_todos_filters_by_status(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"- [ ] open\n- [x] done\n- [-] deferred\n");

        let done = call_tool_json(
            &project,
            "list_todos",
            serde_json::json!({"status": "done"}),
        )
        .await;
        assert_eq!(done["count"], 1);
        assert_eq!(done["results"][0]["status"], "done");

        let all =
            call_tool_json(&project, "list_todos", serde_json::json!({"status": "all"})).await;
        assert_eq!(all["count"], 3);
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_list_todos_empty_points_elsewhere(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("a.md", b"# Just a heading, no tasks.\n");

        let todos = call_tool_json(&project, "list_todos", serde_json::json!({})).await;

        assert_eq!(todos["count"], 0);
        let next = todos["next"].as_str().unwrap_or_default();
        assert!(
            next.contains("search_scraps"),
            "empty todos should point at another way in: {todos}"
        );
    }

    // Called twice so the second read rebuilds the scrap from the stored IR.
    #[rstest]
    #[tokio::test]
    async fn test_frontmatter_tags_are_tags(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap(
            "orders.md",
            b"---\ntags: [sales, a/b]\n---\n\n# Orders\n\n#[[sales]]\n",
        );

        for _ in 0..2 {
            let tags = call_tool_json(&project, "list_tags", serde_json::json!({})).await;
            let text = tags.to_string();
            assert_eq!(text.matches(r#""sales""#).count(), 1, "{text}");
            assert!(
                text.contains(r#""a""#) && text.contains(r#""a/b""#),
                "{text}"
            );

            let backlinks = call_tool_json(
                &project,
                "lookup_tag_backlinks",
                serde_json::json!({"tag": "a"}),
            )
            .await;
            assert!(backlinks.to_string().contains("orders"), "{backlinks}");
        }
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

    /// Test: search_scraps with AND logic - all keywords must match
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
        let mut keys = node.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        assert_eq!(
            keys,
            vec!["ctx", "hop", "key", "title"],
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

    #[rstest]
    #[tokio::test]
    async fn test_reading_a_scrap_points_at_the_neighborhood_map(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("target.md", b"# Target\n\nContent");

        let scrap = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "target"}),
        )
        .await;

        assert!(
            scrap["next"]
                .as_str()
                .unwrap_or_default()
                .contains("lookup_scrap_neighborhood"),
            "a read should name the one-call way to see around the scrap: {scrap}"
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

    // Called twice so the second read rebuilds the scraps from the stored IR.
    #[rstest]
    #[tokio::test]
    async fn test_types_list_and_expand_into_their_scraps(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n");
        project.add_scrap("Shelf/one.md", b"---\ntype: Book\n---\n\n# One\n");
        project.add_scrap("Shelf/two.md", b"---\ntype: Book\n---\n\n# Two\n");
        project.add_scrap("note.md", b"# Note\n\n[[Blog]] #[[Blog]]\n");

        for _ in 0..2 {
            let types = call_tool_json(&project, "list_types", serde_json::json!({})).await;
            assert_eq!(
                types["results"],
                serde_json::json!([
                    {"type": "Book", "scraps_count": 2},
                    {"type": "Blog", "scraps_count": 1},
                ])
            );
            assert_eq!(types["count"], 2);
            assert!(
                types["next"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("lookup_type_scraps"),
                "list_types should say how to expand a type: {types}"
            );

            let blogs = call_tool_json(
                &project,
                "lookup_type_scraps",
                serde_json::json!({"type": "Blog"}),
            )
            .await;
            assert_eq!(
                blogs["results"],
                serde_json::json!([{"key": "post", "title": "post", "ctx": null}])
            );
            assert_eq!(blogs["count"], 1);
            assert!(
                blogs["next"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("get_scrap"),
                "a type's scraps should point at reading one: {blogs}"
            );
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_empty_type_responses_teach_another_way_in(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("note.md", b"# Note\n\n#[[rust]]\n");

        let types = call_tool_json(&project, "list_types", serde_json::json!({})).await;
        assert_eq!(types["count"], 0);
        assert!(
            types["next"]
                .as_str()
                .unwrap_or_default()
                .contains("list_tags"),
            "an untyped wiki should point at the topic map: {types}"
        );

        let scraps = call_tool_json(
            &project,
            "lookup_type_scraps",
            serde_json::json!({"type": "Blog"}),
        )
        .await;
        assert_eq!(scraps["count"], 0);
        assert!(
            scraps["next"]
                .as_str()
                .unwrap_or_default()
                .contains("list_types"),
            "an unused type should point at the types in use: {scraps}"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_projects_the_type_only_when_asked(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap("post.md", b"---\ntype: Blog\n---\n\n# Post\n");
        project.add_scrap("note.md", b"# Note\n");

        let typed = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "post", "fields": ["title", "type"]}),
        )
        .await;
        assert_eq!(typed["type"], "Blog");

        let untyped = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "note", "fields": ["title", "type"]}),
        )
        .await;
        assert!(untyped["type"].is_null() && untyped.as_object().unwrap().contains_key("type"));

        let default =
            call_tool_json(&project, "get_scrap", serde_json::json!({"title": "post"})).await;
        assert!(!default.as_object().unwrap().contains_key("type"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_body_leaves_the_frontmatter_block_out(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "post.md",
            b"---\ntype: Blog\ntags: [a, b]\nowner: alice\n---\n\n\n# Post\n\ntext\n",
        );
        project.add_scrap("open.md", b"---\ntype: Blog\n\n# Open\n");

        let post = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "post", "fields": ["body", "headings"]}),
        )
        .await;
        assert_eq!(post["body"], "# Post\n\ntext\n");
        assert_eq!(post["headings"][0]["line"], 8);

        let open = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "open", "fields": ["body"]}),
        )
        .await;
        assert_eq!(open["body"], "---\ntype: Blog\n\n# Open\n");
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_says_lines_are_file_lines(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tools = list_tools_of(&project).await;
        let get = tools
            .iter()
            .find(|t| t.name.as_ref() == "get_scrap")
            .unwrap();
        let desc = get.description.as_deref().unwrap_or_default();
        assert!(
            desc.contains("frontmatter") && desc.contains("not offsets into body"),
            "get should say what its line numbers count: {desc}"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_projects_verified_only_when_asked(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "orders.md",
            b"---\nverified:\n  by: human:alice\n  at: 2026-10-17T00:00:00Z\n---\n\n# Orders\n",
        );

        let verified = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"key": "orders", "fields": ["verified"]}),
        )
        .await;
        assert_eq!(
            verified["verified"],
            serde_json::json!({
                "tier": "human-reviewed",
                "entries": [{"by": "human:alice", "at": "2026-10-17T00:00:00Z"}]
            })
        );

        let default =
            call_tool_json(&project, "get_scrap", serde_json::json!({"key": "orders"})).await;
        assert!(!default.as_object().unwrap().contains_key("verified"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_projects_the_resource_only_when_asked(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "orders.md",
            b"---\nresource: bigquery://acme/sales/orders\n---\n\n# Orders\n",
        );

        let resource = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"key": "orders", "fields": ["resource"]}),
        )
        .await;
        assert_eq!(resource["resource"], "bigquery://acme/sales/orders");

        let default =
            call_tool_json(&project, "get_scrap", serde_json::json!({"key": "orders"})).await;
        assert!(!default.as_object().unwrap().contains_key("resource"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_projects_sources_only_when_asked(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "orders.md",
            b"---\nsources:\n  resource: https://example.com/orders\n  id: orders-doc\n---\n\n# Orders\n",
        );

        let sources = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"key": "orders", "fields": ["sources"]}),
        )
        .await;
        assert_eq!(
            sources["sources"],
            serde_json::json!([{"resource": "https://example.com/orders", "id": "orders-doc"}])
        );

        let default =
            call_tool_json(&project, "get_scrap", serde_json::json!({"key": "orders"})).await;
        assert!(!default.as_object().unwrap().contains_key("sources"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_projects_the_description_only_when_asked(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "orders.md",
            b"---\ndescription: One row per completed customer order.\n---\n\n# Orders\n",
        );
        project.add_scrap("note.md", b"# Note\n");

        let described = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "orders", "fields": ["title", "description"]}),
        )
        .await;
        assert_eq!(
            described["description"],
            "One row per completed customer order."
        );

        let plain = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "note", "fields": ["title", "description"]}),
        )
        .await;
        assert!(plain["description"].is_null());
        assert!(plain.as_object().unwrap().contains_key("description"));

        let default = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "orders"}),
        )
        .await;
        assert!(!default.as_object().unwrap().contains_key("description"));
    }

    #[rstest]
    #[tokio::test]
    async fn test_search_scraps_carries_a_description_only_where_there_is_one(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        project.add_scrap(
            "orders.md",
            b"---\ndescription: One row per completed customer order.\n---\n\n# Orders\n",
        );
        project.add_scrap("orders log.md", b"# Orders log\n");

        let response = call_tool_json(
            &project,
            "search_scraps",
            serde_json::json!({"query": "orders"}),
        )
        .await;
        let results = response["results"].as_array().unwrap();
        let hit = |title: &str| results.iter().find(|r| r["title"] == title).unwrap();

        assert_eq!(
            hit("orders")["description"],
            "One row per completed customer order."
        );
        assert_eq!(
            hit("orders log"),
            &serde_json::json!({"key": "orders log", "title": "orders log", "ctx": null})
        );
    }

    #[rstest]
    fn test_instructions_name_the_type_tools(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let instructions = server.get_info().instructions.unwrap();

        assert!(
            instructions.contains("list_types") && instructions.contains("lookup_type_scraps"),
            "the flow should name the type tools: {instructions}"
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

    async fn call_tool_error(
        project: &TempScrapProject,
        name: &str,
        arguments: serde_json::Value,
    ) -> String {
        let server = ScrapsServer::new(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
        );
        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });
        let client = ().serve(client_stream).await.unwrap();

        let params = CallToolRequestParams::new(name.to_string())
            .with_arguments(arguments.as_object().unwrap().clone());
        let error = client.call_tool(params).await.unwrap_err();

        client.cancel().await.unwrap();
        server_handle.abort();
        error.to_string()
    }

    const SCRAP_TOOLS: [&str; 4] = [
        "get_scrap",
        "lookup_scrap_links",
        "lookup_scrap_backlinks",
        "lookup_scrap_neighborhood",
    ];

    fn add_titled_wiki(project: &TempScrapProject) {
        project.add_scrap(
            "tables/orders.md",
            b"---\ntitle: Customer Orders\n---\n\nOne row per order.\n",
        );
        project.add_scrap(
            "metrics/revenue.md",
            b"See [[tables/orders]] and [[tables/orders|the orders table]].\n",
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_scrap_takes_a_key_and_returns_key_title_and_ctx(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        add_titled_wiki(&project);

        let by_key = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"key": "tables/orders"}),
        )
        .await;
        assert_eq!(by_key["key"], "tables/orders");
        assert_eq!(by_key["title"], "Customer Orders");
        assert_eq!(by_key["ctx"], "tables");

        let by_stem = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"title": "orders", "ctx": "tables"}),
        )
        .await;
        assert_eq!(by_stem, by_key);
    }

    #[rstest]
    #[tokio::test]
    async fn test_the_older_title_stays_a_file_stem(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        add_titled_wiki(&project);

        for name in SCRAP_TOOLS {
            let error = call_tool_error(
                &project,
                name,
                serde_json::json!({"title": "Customer Orders", "ctx": "tables"}),
            )
            .await;
            assert!(error.contains("Scrap not found"), "{name}: {error}");
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrap_tools_require_exactly_one_of_key_and_title(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        add_titled_wiki(&project);

        for name in SCRAP_TOOLS {
            let neither = call_tool_error(&project, name, serde_json::json!({"depth": 1})).await;
            assert!(neither.contains("Missing `key`"), "{name}: {neither}");

            let both = call_tool_error(
                &project,
                name,
                serde_json::json!({"key": "tables/orders", "title": "orders"}),
            )
            .await;
            assert!(both.contains("not both"), "{name}: {both}");
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_a_search_hit_is_found_by_title_and_its_key_reaches_the_scrap(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        add_titled_wiki(&project);

        let response = call_tool_json(
            &project,
            "search_scraps",
            serde_json::json!({"query": "Customer"}),
        )
        .await;
        let hit = &response["results"][0];
        assert_eq!(hit["key"], "tables/orders");
        assert_eq!(hit["title"], "Customer Orders");
        assert_eq!(hit["ctx"], "tables");

        let read = call_tool_json(
            &project,
            "get_scrap",
            serde_json::json!({"key": hit["key"]}),
        )
        .await;
        assert_eq!(read["title"], "Customer Orders");
    }

    #[rstest]
    #[tokio::test]
    async fn test_relations_name_each_scrap_by_key_and_title(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        add_titled_wiki(&project);
        let orders = serde_json::json!({"key": "tables/orders", "title": "Customer Orders", "ctx": "tables"});
        let revenue =
            serde_json::json!({"key": "metrics/revenue", "title": "revenue", "ctx": "metrics"});

        let links = call_tool_json(
            &project,
            "lookup_scrap_links",
            serde_json::json!({"key": "metrics/revenue"}),
        )
        .await;
        assert_eq!(links["count"], 2);
        assert_eq!(links["results"][0]["key"], orders["key"]);
        assert_eq!(links["results"][0]["title"], orders["title"]);

        let backlinks = call_tool_json(
            &project,
            "lookup_scrap_backlinks",
            serde_json::json!({"key": "tables/orders"}),
        )
        .await;
        let mut revenue_backlink = revenue.clone();
        revenue_backlink["kinds"] = serde_json::json!(["link"]);
        assert_eq!(backlinks["results"], serde_json::json!([revenue_backlink]));

        let map = call_tool_json(
            &project,
            "lookup_scrap_neighborhood",
            serde_json::json!({"key": "metrics/revenue"}),
        )
        .await;
        assert_eq!(map["nodes"][1]["key"], orders["key"]);
        assert_eq!(map["nodes"][1]["title"], orders["title"]);
        assert_eq!(
            map["edges"],
            serde_json::json!([{"from": revenue, "to": orders}])
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_scrap_tool_schemas_offer_key_and_the_older_pair(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let tools = list_tools_of(&project).await;
        for name in SCRAP_TOOLS {
            let tool = tools.iter().find(|t| t.name.as_ref() == name).unwrap();
            let schema = serde_json::to_value(&*tool.input_schema).unwrap();
            let properties = schema["properties"].as_object().unwrap();
            let described = properties["key"]["description"].as_str().unwrap();
            assert!(described.contains("Pass the `key`"), "{name}: {described}");
            let older = properties["title"]["description"].as_str().unwrap();
            assert!(older.contains("Still accepted"), "{name}: {older}");
            assert!(
                schema
                    .get("required")
                    .is_none_or(|r| r.as_array().unwrap().is_empty()),
                "{name} should require neither key nor title by schema: {schema}"
            );
        }
    }
}
