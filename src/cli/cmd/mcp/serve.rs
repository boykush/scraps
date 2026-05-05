use std::path::Path;

use crate::{
    cli::config::scrap_config::ScrapConfig,
    cli::path_resolver::PathResolver,
    error::{McpError, ScrapsResult},
    mcp::server::ScrapsServer,
};
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub async fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| McpError::TracingSetup(e.to_string()))?;

    info!("Starting Scraps MCP server...");

    // Set up path resolver. The wiki root is the directory containing
    // `.scraps.toml` (i.e. the project root). Config is loaded only to resolve
    // the configured `output_dir` so it can be excluded from scrap traversal.
    let path_resolver = PathResolver::new(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to resolve paths: {e}")))?;
    let config = ScrapConfig::from_path(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to load config: {e}")))?;

    let scraps_dir = path_resolver.scraps_dir();
    let exclude_dirs = vec![
        path_resolver.static_dir(),
        path_resolver.output_dir(&config),
    ];

    let service = ScrapsServer::new(scraps_dir, exclude_dirs)
        .serve((stdin(), stdout()))
        .await
        .inspect_err(|e| {
            tracing::error!("Failed to start Scraps MCP server: {}", e);
        })
        .map_err(|e| McpError::ServiceError(e.to_string()))?;

    service
        .waiting()
        .await
        .map_err(|e| McpError::ServiceError(e.to_string()))?;
    Ok(())
}
