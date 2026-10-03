use std::path::{Path, PathBuf};

use crate::{
    cli::config::scrap_config::ScrapConfig,
    cli::path_resolver::PathResolver,
    error::{McpError, ScrapsResult},
    mcp::{
        self,
        server::ScrapsServer,
        telemetry::Telemetry,
        traced::{SpanOptions, Traced},
    },
};
use rmcp::ServiceExt;
use tokio::io::{stdin, stdout};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

pub async fn run(
    project_path: Option<&Path>,
    http_addr: Option<&str>,
    allowed_hosts: Vec<String>,
) -> ScrapsResult<()> {
    let telemetry = init_tracing()?;
    let spans = telemetry.as_ref().map(Telemetry::span_options);

    let served = serve(project_path, http_addr, allowed_hosts, spans).await;

    // `main` drops the runtime the exporter runs on as soon as this returns.
    if let Some(telemetry) = telemetry {
        telemetry.shutdown().await;
    }
    served
}

/// Logs go to stderr because the stdio transport owns stdout for JSON-RPC.
/// Spans are exported as well when the environment asks for it.
fn init_tracing() -> ScrapsResult<Option<Telemetry>> {
    let logs = || {
        tracing_subscriber::registry()
            .with(LevelFilter::INFO)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
    };

    // Telemetry says what it makes of the environment through the logs, which
    // cannot be installed for good until its layer exists.
    let telemetry = tracing::subscriber::with_default(logs(), Telemetry::from_env);
    let subscriber = logs().with(telemetry.as_ref().map(Telemetry::layer));
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| McpError::TracingSetup(e.to_string()))?;

    Ok(telemetry)
}

async fn serve(
    project_path: Option<&Path>,
    http_addr: Option<&str>,
    allowed_hosts: Vec<String>,
    spans: Option<SpanOptions>,
) -> ScrapsResult<()> {
    let (scraps_dir, exclude_dirs) = resolve_dirs(project_path)?;

    match http_addr {
        Some(addr) => serve_http(addr, scraps_dir, exclude_dirs, allowed_hosts, spans).await,
        None => serve_stdio(scraps_dir, exclude_dirs, spans).await,
    }
}

/// Resolve the wiki root. The root is the directory containing `.scraps.toml`
/// (i.e. the project root). Config is loaded only to resolve the configured
/// `output_dir` so it can be excluded from scrap traversal.
fn resolve_dirs(project_path: Option<&Path>) -> ScrapsResult<(PathBuf, Vec<PathBuf>)> {
    let path_resolver = PathResolver::new(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to resolve paths: {e}")))?;
    let config = ScrapConfig::from_path(project_path)
        .map_err(|e| McpError::ServiceError(format!("Failed to load config: {e}")))?;

    let exclude_dirs = vec![
        path_resolver.static_dir(),
        path_resolver.output_dir(&config),
    ];

    Ok((path_resolver.scraps_dir(), exclude_dirs))
}

async fn serve_stdio(
    scraps_dir: PathBuf,
    exclude_dirs: Vec<PathBuf>,
    spans: Option<SpanOptions>,
) -> ScrapsResult<()> {
    info!("Starting Scraps MCP server...");

    let service = Traced::new(ScrapsServer::new(scraps_dir, exclude_dirs), spans)
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

async fn serve_http(
    addr: &str,
    scraps_dir: PathBuf,
    exclude_dirs: Vec<PathBuf>,
    allowed_hosts: Vec<String>,
    spans: Option<SpanOptions>,
) -> ScrapsResult<()> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| McpError::Bind(format!("{addr}: {e}")))?;

    // Report the bound address rather than the requested one: a hostname can
    // resolve to an address family the client does not use.
    let local_addr = listener
        .local_addr()
        .map_err(|e| McpError::Bind(format!("{addr}: {e}")))?;
    info!(
        "Scraps MCP server listening on http://{local_addr}{} (wiki: {})",
        mcp::http::ENDPOINT_PATH,
        scraps_dir.display()
    );

    let service = mcp::http::build_service(scraps_dir, exclude_dirs, allowed_hosts, spans);
    tokio::select! {
        result = mcp::http::serve(listener, service) => {
            result.map_err(|e| McpError::ServiceError(e.to_string()))?;
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutting down Scraps MCP server");
        }
    }
    Ok(())
}
