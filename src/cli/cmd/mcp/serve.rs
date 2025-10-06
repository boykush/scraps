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

    // Set up path resolver and config
    let path_resolver = PathResolver::new(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to resolve paths: {e}")))?;

    // Load config to get base_url
    let config = ScrapConfig::from_path(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to load config: {e}")))?;

    let scraps_dir = path_resolver.scraps_dir(&config);

    let base_url = config.base_url.into_base_url();

    let service = ScrapsServer::new(scraps_dir, base_url)
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
