//! One `tracing` span per MCP request, of the kind `scraps build -v` times its
//! stages with, named and attributed by the MCP semantic conventions:
//! <https://github.com/open-telemetry/semantic-conventions-genai/blob/main/docs/gen-ai/mcp.md>
//! `telemetry` holds the layer that exports them.

use std::borrow::Cow;

use hyper::http::Version;
use hyper::http::request::Parts;
use opentelemetry::propagation::{Extractor, TextMapPropagator};
use opentelemetry::trace::Status;
use opentelemetry::{Context, Value};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use rmcp::model::{
    CallToolRequestMethod, CallToolRequestParams, CallToolResponse, ConstString,
    DiscoverRequestMethod, DiscoverResult, ErrorCode, InitializeRequestParams, InitializeResult,
    InitializeResultMethod, ListToolsRequestMethod, ListToolsResult, PaginatedRequestParams,
    PingRequestMethod, ProtocolVersion, RequestMetaObject, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use tracing::{Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// JSON-RPC codes for a request the server could not serve as sent. The
/// conventions record the code but keep these out of the error count.
const CALLER_MISTAKES: [ErrorCode; 5] = [
    ErrorCode::PARSE_ERROR,
    ErrorCode::INVALID_REQUEST,
    ErrorCode::METHOD_NOT_FOUND,
    ErrorCode::INVALID_PARAMS,
    ErrorCode::RESOURCE_NOT_FOUND,
];

/// What a traced server's spans may carry.
#[derive(Clone, Copy)]
pub struct SpanOptions {
    /// Lets spans carry what the caller sent: tool arguments, and error
    /// messages, which echo the arguments they complain about.
    pub capture_content: bool,
}

/// An MCP server that reports each request it answers as a server span.
///
/// It forwards what a tools-only server answers; a server that grows prompts
/// or resources has to be forwarded here as well.
pub struct Traced<S> {
    inner: S,
    spans: Option<SpanOptions>,
}

impl<S> Traced<S> {
    /// Without options no span is opened and every request passes straight
    /// through.
    pub fn new(inner: S, spans: Option<SpanOptions>) -> Self {
        Self { inner, spans }
    }

    fn span(
        &self,
        method: &'static str,
        tool: Option<&CallToolRequestParams>,
        context: &RequestContext<RoleServer>,
    ) -> Span {
        let Some(options) = self.spans else {
            return Span::none();
        };

        let name = match tool {
            Some(tool) => Cow::Owned(format!("{method} {}", tool.name)),
            None => Cow::Borrowed(method),
        };
        // A `tracing` span's own name is fixed at compile time; these two
        // fields are how it hands the exporter its name and kind.
        let span = tracing::info_span!("mcp_request", otel.name = &*name, otel.kind = "server");
        // Fails only without an exporting layer, where there is no span to parent.
        let _ = span.set_parent(caller_context(&context.meta));

        // Set as attributes rather than fields, which would repeat every one
        // of them in front of each log line written during the request.
        span.set_attribute("mcp.method.name", method);
        span.set_attribute("jsonrpc.request.id", context.id.to_string());
        if let Some(version) = context.protocol_version() {
            span.set_attribute("mcp.protocol.version", version.to_string());
        }
        set_network_attributes(&span, context);

        if let Some(tool) = tool {
            span.set_attribute("gen_ai.operation.name", "execute_tool");
            span.set_attribute("gen_ai.tool.name", tool.name.to_string());
            // Span attributes cannot hold a map, so the conventions' fallback applies: JSON.
            if options.capture_content
                && let Some(arguments) = &tool.arguments
                && let Ok(json) = serde_json::to_string(arguments)
            {
                span.set_attribute("gen_ai.tool.call.arguments", json);
            }
        }

        span
    }

    fn fail(&self, span: &Span, error_type: impl Into<Value>, message: &str) {
        span.set_attribute("error.type", error_type);
        let captures = self.spans.is_some_and(|options| options.capture_content);
        let description = if captures { message } else { "" };
        span.set_status(Status::error(description.to_string()));
    }

    /// Record how the request ended on its span and hand the result back.
    fn finish<T>(&self, span: Span, result: Result<T, ErrorData>) -> Result<T, ErrorData> {
        if let Err(error) = &result {
            let code = error.code.0.to_string();
            span.set_attribute("rpc.response.status_code", code.clone());
            if !CALLER_MISTAKES.contains(&error.code) {
                self.fail(&span, code, &error.message);
            }
        }
        result
    }
}

/// rmcp attaches the HTTP request to the context on the Streamable HTTP
/// transport only; the other transport scraps serves is stdio.
fn set_network_attributes(span: &Span, context: &RequestContext<RoleServer>) {
    let Some(http) = context.extensions.get::<Parts>() else {
        span.set_attribute("network.transport", "pipe");
        return;
    };
    // The listener speaks HTTP/1 only.
    let version = match http.version {
        Version::HTTP_10 => "1.0",
        _ => "1.1",
    };
    span.set_attribute("network.transport", "tcp");
    span.set_attribute("network.protocol.name", "http");
    span.set_attribute("network.protocol.version", version);
}

/// The caller's trace, when it sent one. MCP carries W3C Trace Context in
/// `params._meta` rather than in transport headers, so that it survives stdio.
fn caller_context(meta: &RequestMetaObject) -> Context {
    TraceContextPropagator::new().extract_with_context(&Context::new(), &MetaExtractor(meta))
}

struct MetaExtractor<'a>(&'a RequestMetaObject);

impl Extractor for MetaExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(String::as_str).collect()
    }
}

impl<S: ServerHandler> ServerHandler for Traced<S> {
    fn get_info(&self) -> ServerConfig {
        self.inner.get_info()
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        self.inner.supported_protocol_versions()
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.inner.get_tool(name)
    }

    async fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, ErrorData> {
        let span = self.span(InitializeResultMethod::VALUE, None, &context);
        let result = self
            .inner
            .initialize(request, context)
            .instrument(span.clone())
            .await;
        self.finish(span, result)
    }

    async fn discover(
        &self,
        context: RequestContext<RoleServer>,
    ) -> Result<DiscoverResult, ErrorData> {
        let span = self.span(DiscoverRequestMethod::VALUE, None, &context);
        let result = self.inner.discover(context).instrument(span.clone()).await;
        self.finish(span, result)
    }

    async fn ping(&self, context: RequestContext<RoleServer>) -> Result<(), ErrorData> {
        let span = self.span(PingRequestMethod::VALUE, None, &context);
        let result = self.inner.ping(context).instrument(span.clone()).await;
        self.finish(span, result)
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let span = self.span(ListToolsRequestMethod::VALUE, None, &context);
        let result = self
            .inner
            .list_tools(request, context)
            .instrument(span.clone())
            .await;
        self.finish(span, result)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let span = self.span(CallToolRequestMethod::VALUE, Some(&request), &context);
        let result = self
            .inner
            .call_tool(request, context)
            .instrument(span.clone())
            .await;
        // A tool reports its own failure inside a successful JSON-RPC response.
        if let Ok(CallToolResponse::Complete(tool_result)) = &result
            && tool_result.is_error == Some(true)
        {
            self.fail(&span, "tool_error", "");
        }
        self.finish(span, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::server::ScrapsServer;
    use crate::mcp::telemetry::capture_spans;
    use crate::test_fixtures::{TempScrapProject, temp_scrap_project};
    use opentelemetry_sdk::trace::SpanData;
    use rmcp::ServiceExt;
    use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities};
    use rstest::rstest;

    /// A server whose only tool reports its own failure.
    struct FailingTool;

    impl ServerHandler for FailingTool {
        fn get_info(&self) -> ServerConfig {
            ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
        }

        async fn call_tool(
            &self,
            _request: CallToolRequestParams,
            _context: RequestContext<RoleServer>,
        ) -> Result<CallToolResponse, ErrorData> {
            Ok(CallToolResult::error(vec![ContentBlock::text("no such scrap")]).into())
        }
    }

    async fn span_of_a_call_to(server: impl ServerHandler) -> SpanData {
        let (exporter, _subscriber) = capture_spans();
        let spans = SpanOptions {
            capture_content: false,
        };

        let (client_stream, server_stream) = tokio::io::duplex(4096);
        let server = Traced::new(server, Some(spans));
        let server_handle = tokio::spawn(async move { server.serve(server_stream).await });
        let client = ().serve(client_stream).await.unwrap();
        client
            .call_tool(CallToolRequestParams::new("broken"))
            .await
            .unwrap();
        client.cancel().await.unwrap();
        server_handle.abort();

        exporter
            .get_finished_spans()
            .unwrap()
            .into_iter()
            .find(|span| span.name == "tools/call broken")
            .expect("the tool call should have left a span")
    }

    fn attribute(span: &SpanData, key: &str) -> Option<String> {
        span.attributes
            .iter()
            .find(|attribute| attribute.key.as_str() == key)
            .map(|attribute| attribute.value.to_string())
    }

    /// `Traced` forwards what a tools-only server answers, so anything else
    /// `ScrapsServer` starts to advertise has to be forwarded there first.
    #[rstest]
    fn test_scraps_server_advertises_only_what_traced_forwards(
        #[from(temp_scrap_project)] project: TempScrapProject,
    ) {
        let server = ScrapsServer::new(project.scraps_dir.clone(), vec![]);

        assert_eq!(
            server.get_info().capabilities,
            ServerCapabilities::builder().enable_tools().build()
        );
    }

    #[tokio::test]
    async fn test_tool_reported_failure_marks_the_span_as_error() {
        let span = span_of_a_call_to(FailingTool).await;

        assert_eq!(
            attribute(&span, "error.type").as_deref(),
            Some("tool_error")
        );
        assert_eq!(attribute(&span, "rpc.response.status_code"), None);
        assert_eq!(span.status, Status::error(""));
    }

    #[tokio::test]
    async fn test_span_outside_http_records_a_pipe_transport() {
        let span = span_of_a_call_to(FailingTool).await;

        assert_eq!(
            attribute(&span, "network.transport").as_deref(),
            Some("pipe")
        );
        assert_eq!(attribute(&span, "network.protocol.name"), None);
    }
}
