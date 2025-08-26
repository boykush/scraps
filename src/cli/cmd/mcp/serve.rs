use std::path::Path;

use crate::{
    cli::cmd::mcp::server::ScrapsServer,
    error::{McpError, ScrapsResult},
};
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub async fn run(project_path: Option<&Path>) -> ScrapsResult<()> {
    let _ = project_path;

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| McpError::TracingSetup(e.to_string()))?;

    info!("Starting Scraps MCP server...");

    let service = ScrapsServer::new()
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
