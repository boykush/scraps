use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;

use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::net::TcpListener;

use crate::mcp::server::ScrapsServer;

/// Path the MCP endpoint is served at, relative to the bind address.
pub const ENDPOINT_PATH: &str = "/mcp";

type McpService = StreamableHttpService<ScrapsServer, NeverSessionManager>;

/// Build the MCP service in stateless mode, so one long-running process can
/// back any number of clients without retaining per-client sessions. The cost
/// is server-initiated notifications, which the read-only tools never send.
pub fn build_service(
    scraps_dir: PathBuf,
    exclude_dirs: Vec<PathBuf>,
    allowed_hosts: Vec<String>,
) -> McpService {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true);

    // Extend rmcp's loopback default instead of replacing it, so a port-forward
    // or local curl still reaches a server published under a public hostname.
    // The list is never empty, which rmcp would read as "allow any host".
    let allowed_hosts = config
        .allowed_hosts
        .iter()
        .cloned()
        .chain(allowed_hosts)
        .collect::<Vec<_>>();
    let config = config.with_allowed_hosts(allowed_hosts);

    StreamableHttpService::new(
        move || Ok(ScrapsServer::new(scraps_dir.clone(), exclude_dirs.clone())),
        Arc::new(NeverSessionManager::default()),
        config,
    )
}

pub async fn serve(listener: TcpListener, service: McpService) -> std::io::Result<()> {
    let service = Arc::new(service);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let service = service.clone();

        tokio::spawn(async move {
            let handler = service_fn(move |request| route(service.clone(), request));
            if let Err(err) = http1::Builder::new().serve_connection(io, handler).await {
                tracing::debug!("Failed to serve MCP connection: {err:?}");
            }
        });
    }
}

/// rmcp handles any path it is given, so the endpoint is routed here.
async fn route(
    service: Arc<McpService>,
    request: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Infallible> {
    if request.uri().path() != ENDPOINT_PATH {
        return Ok(not_found());
    }

    Ok(service.handle(request).await)
}

fn not_found() -> Response<BoxBody<Bytes, Infallible>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from_static(b"Not Found")).boxed())
        .expect("valid response")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{temp_scrap_project, TempScrapProject};
    use http_body_util::{BodyExt, Full};
    use hyper::header::{ACCEPT, CONTENT_TYPE, HOST};
    use hyper::{Method, Request, StatusCode};
    use hyper_util::rt::TokioIo;
    use rstest::rstest;
    use std::net::SocketAddr;
    use tokio::net::TcpStream;
    use tokio::task::JoinHandle;

    async fn spawn_server(
        project: &TempScrapProject,
    ) -> (SocketAddr, JoinHandle<std::io::Result<()>>) {
        spawn_server_with_allowed_hosts(project, vec![]).await
    }

    async fn spawn_server_with_allowed_hosts(
        project: &TempScrapProject,
        allowed_hosts: Vec<String>,
    ) -> (SocketAddr, JoinHandle<std::io::Result<()>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let service = build_service(
            project.scraps_dir.clone(),
            vec![project.static_dir.clone(), project.output_dir.clone()],
            allowed_hosts,
        );

        (addr, tokio::spawn(serve(listener, service)))
    }

    async fn post(addr: SocketAddr, path: &str, body: serde_json::Value) -> (StatusCode, String) {
        post_with_host(addr, path, &addr.to_string(), body).await
    }

    async fn post_with_host(
        addr: SocketAddr,
        path: &str,
        host: &str,
        body: serde_json::Value,
    ) -> (StatusCode, String) {
        let stream = TcpStream::connect(addr).await.unwrap();
        let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .unwrap();
        tokio::spawn(connection);

        let request = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(HOST, host)
            .header(CONTENT_TYPE, "application/json")
            .header(ACCEPT, "application/json, text/event-stream")
            .body(Full::new(Bytes::from(body.to_string())))
            .unwrap();

        let response = sender.send_request(request).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[rstest]
    #[tokio::test]
    async fn test_list_tools(#[from(temp_scrap_project)] project: TempScrapProject) {
        let (addr, server_handle) = spawn_server(&project).await;

        let (status, body) = post(
            addr,
            ENDPOINT_PATH,
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 6);

        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_initialize(#[from(temp_scrap_project)] project: TempScrapProject) {
        let (addr, server_handle) = spawn_server(&project).await;

        let (status, body) = post(
            addr,
            ENDPOINT_PATH,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": serde_json::to_value(rmcp::model::ClientInfo::default()).unwrap(),
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(response["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("Scraps wiki"));
        assert!(response["result"]["capabilities"]["tools"].is_object());

        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_call_search_scraps(#[from(temp_scrap_project)] project: TempScrapProject) {
        project.add_scrap("test.md", b"# Test Scrap\n\nContent here");

        let (addr, server_handle) = spawn_server(&project).await;

        let (status, body) = post(
            addr,
            ENDPOINT_PATH,
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {"name": "search_scraps", "arguments": {"query": "test"}},
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let response: serde_json::Value = serde_json::from_str(&body).unwrap();
        let text = response["result"]["content"][0]["text"].as_str().unwrap();
        let result: serde_json::Value = serde_json::from_str(text).unwrap();
        assert!(result["count"].as_u64().unwrap() > 0);

        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_unconfigured_host_is_forbidden(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let (addr, server_handle) = spawn_server(&project).await;

        let (status, _) = post_with_host(
            addr,
            ENDPOINT_PATH,
            "mcp.example.com",
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);

        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_configured_host_is_allowed(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let (addr, server_handle) =
            spawn_server_with_allowed_hosts(&project, vec!["mcp.example.com".to_string()]).await;

        let (status, _) = post_with_host(
            addr,
            ENDPOINT_PATH,
            "mcp.example.com",
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::OK);

        server_handle.abort();
    }

    /// The flag extends the loopback default instead of replacing it, so a
    /// port-forward keeps reaching a server published under a public hostname.
    #[rstest]
    #[tokio::test]
    async fn test_loopback_still_allowed_alongside_configured_host(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let (addr, server_handle) =
            spawn_server_with_allowed_hosts(&project, vec!["mcp.example.com".to_string()]).await;

        let (status, _) = post_with_host(
            addr,
            ENDPOINT_PATH,
            &addr.to_string(),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::OK);

        server_handle.abort();
    }

    #[rstest]
    #[tokio::test]
    async fn test_unknown_path_returns_not_found(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let (addr, server_handle) = spawn_server(&project).await;

        let (status, _) = post(
            addr,
            "/",
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND);

        server_handle.abort();
    }
}
