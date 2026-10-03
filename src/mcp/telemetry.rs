//! OTLP export of the `tracing` spans scraps opens. It is configured the way
//! any OpenTelemetry SDK is, through the `OTEL_*` environment variables, and
//! stays off unless one of them names an endpoint.

use std::time::Duration;

use opentelemetry::InstrumentationScope;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{
    OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_TRACES_ENDPOINT, SpanExporter,
};
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use tracing::Subscriber;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::registry::LookupSpan;
use url::Url;

use crate::mcp::traced::SpanOptions;

/// The opt-in the GenAI semantic conventions name for attributes that carry
/// what a user sent.
const CAPTURE_CONTENT: &str = "OTEL_INSTRUMENTATION_GENAI_CAPTURE_MESSAGE_CONTENT";

/// What the SDK allows a shutdown by default; its Tokio batch processor does
/// not apply the limit itself.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Telemetry {
    provider: SdkTracerProvider,
    capture_content: bool,
}

impl Telemetry {
    /// `None` unless the environment names an OTLP endpoint. A setting that
    /// cannot be honoured is logged and leaves tracing off: scraps may have
    /// inherited it from a shell that set it for another program.
    pub fn from_env() -> Option<Self> {
        let var = |name: &str| std::env::var(name).ok();
        let endpoint = otlp_endpoint(var)?;

        // The exporter would only ever report the failed connection as a
        // "network error", once per batch.
        if Url::parse(&endpoint).is_ok_and(|url| url.scheme() == "https") {
            tracing::warn!(
                "Not exporting traces: {endpoint} needs TLS, which scraps is built without. \
                 Point it at a collector over http:// instead."
            );
            return None;
        }

        let exporter = SpanExporter::builder()
            .with_http()
            .build()
            .inspect_err(|e| tracing::warn!("Not exporting traces: {e}"))
            .ok()?;

        tracing::info!("Exporting traces over OTLP/HTTP to {endpoint}");
        Some(Self::new(exporter, captures_content(var)))
    }

    fn new(exporter: SpanExporter, capture_content: bool) -> Self {
        // The SDK's default batch processor exports from a thread of its own,
        // where the exporter's hyper client has no Tokio runtime to run on.
        let processor = BatchSpanProcessor::builder(exporter, runtime::Tokio).build();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .build();

        Self {
            provider,
            capture_content,
        }
    }

    /// The layer a `tracing` subscriber needs for the spans to be exported.
    pub fn layer<S>(&self) -> impl Layer<S> + use<S>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        let scope = InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
            .with_version(env!("CARGO_PKG_VERSION"))
            .build();

        exported_spans(self.provider.tracer_with_scope(scope))
    }

    pub fn span_options(&self) -> SpanOptions {
        SpanOptions {
            capture_content: self.capture_content,
        }
    }

    /// Export the spans still buffered. The processor answers from a runtime
    /// task while its caller blocks, so the wait is kept off the async threads.
    pub async fn shutdown(self) {
        let provider = self.provider;
        let flush = tokio::task::spawn_blocking(move || provider.shutdown());

        match tokio::time::timeout(SHUTDOWN_TIMEOUT, flush).await {
            Ok(Ok(Ok(()))) => {}
            Ok(Ok(Err(e))) => tracing::warn!("Failed to export the remaining traces: {e}"),
            Ok(Err(e)) => tracing::warn!("Failed to export the remaining traces: {e}"),
            Err(_) => tracing::warn!("Timed out exporting the remaining traces"),
        }
    }
}

/// Every crate in the process reports to the one subscriber, so the filter is
/// what keeps the spans rmcp opens for itself, and every log line, out of the
/// traces. The layer's own extras, such as the source location of a span, are
/// off so that a span carries only what the code that opened it set.
fn exported_spans<S>(tracer: SdkTracer) -> impl Layer<S> + use<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_location(false)
        .with_target(false)
        .with_threads(false)
        .with_tracked_inactivity(false)
        .with_filter(filter_fn(|metadata| {
            metadata.is_span() && metadata.target().starts_with(env!("CARGO_CRATE_NAME"))
        }))
}

/// Export the spans this thread opens into memory as each one closes, so a
/// test can read them back as soon as it has the response.
#[cfg(test)]
pub(crate) fn capture_spans() -> (
    opentelemetry_sdk::trace::InMemorySpanExporter,
    tracing::subscriber::DefaultGuard,
) {
    use tracing_subscriber::layer::SubscriberExt;

    let exporter = opentelemetry_sdk::trace::InMemorySpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let subscriber = tracing_subscriber::registry().with(exported_spans(provider.tracer("test")));

    (exporter, tracing::subscriber::set_default(subscriber))
}

/// The endpoint the exporter will resolve, by the OTLP rule: the traces
/// variable wins over the general one, and an empty value counts as unset.
fn otlp_endpoint(var: impl Fn(&str) -> Option<String>) -> Option<String> {
    [
        OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
        OTEL_EXPORTER_OTLP_ENDPOINT,
    ]
    .into_iter()
    .find_map(|name| var(name).filter(|endpoint| !endpoint.is_empty()))
}

fn captures_content(var: impl Fn(&str) -> Option<String>) -> bool {
    var(CAPTURE_CONTENT).is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::{BodyExt, Full};
    use hyper::body::{Bytes, Incoming};
    use hyper::server::conn::http1;
    use hyper::service::service_fn;
    use hyper::{Request, Response};
    use hyper_util::rt::TokioIo;
    use opentelemetry::trace::{Span, Tracer};
    use opentelemetry_otlp::WithExportConfig;
    use rstest::rstest;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;

    fn env(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let vars: Vec<(String, String)> = vars
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        move |name| {
            vars.iter()
                .find(|(candidate, _)| candidate == name)
                .map(|(_, value)| value.clone())
        }
    }

    #[rstest]
    #[case::unset(&[], None)]
    #[case::empty_counts_as_unset(&[("OTEL_EXPORTER_OTLP_ENDPOINT", "")], None)]
    #[case::general(
        &[("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318")],
        Some("http://collector:4318")
    )]
    #[case::traces(
        &[("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://collector:4318/v1/traces")],
        Some("http://collector:4318/v1/traces")
    )]
    #[case::traces_wins(
        &[
            ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://collector:4318"),
            ("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://traces:4318/v1/traces"),
        ],
        Some("http://traces:4318/v1/traces")
    )]
    fn test_otlp_endpoint(#[case] vars: &[(&str, &str)], #[case] expected: Option<&str>) {
        assert_eq!(otlp_endpoint(env(vars)).as_deref(), expected);
    }

    #[rstest]
    #[case::unset(&[], false)]
    #[case::enabled(&[(CAPTURE_CONTENT, "true")], true)]
    #[case::any_case(&[(CAPTURE_CONTENT, "TRUE")], true)]
    #[case::disabled(&[(CAPTURE_CONTENT, "false")], false)]
    #[case::not_a_boolean(&[(CAPTURE_CONTENT, "1")], false)]
    fn test_captures_content(#[case] vars: &[(&str, &str)], #[case] expected: bool) {
        assert_eq!(captures_content(env(vars)), expected);
    }

    /// An OTLP/HTTP receiver that hands each request it accepts to the test.
    async fn spawn_collector() -> (SocketAddr, mpsc::UnboundedReceiver<(String, Bytes)>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (received, requests) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.unwrap();
                let received = received.clone();
                tokio::spawn(async move {
                    let handler = service_fn(move |request: Request<Incoming>| {
                        let received = received.clone();
                        async move {
                            let path = request.uri().path().to_string();
                            let body = request.into_body().collect().await?.to_bytes();
                            let _ = received.send((path, body));
                            Ok::<_, hyper::Error>(Response::new(Full::new(Bytes::new())))
                        }
                    });
                    let _ = http1::Builder::new()
                        .serve_connection(TokioIo::new(stream), handler)
                        .await;
                });
            }
        });

        (addr, requests)
    }

    /// Batches go out every few seconds, so a span that has just ended reaches
    /// the collector before the process exits only if shutdown exports it.
    #[tokio::test(flavor = "multi_thread")]
    async fn test_shutdown_exports_buffered_spans() {
        let (addr, mut requests) = spawn_collector().await;
        let exporter = SpanExporter::builder()
            .with_http()
            .with_endpoint(format!("http://{addr}/v1/traces"))
            .build()
            .unwrap();
        let telemetry = Telemetry::new(exporter, false);

        telemetry.provider.tracer("test").start("buffered").end();
        telemetry.shutdown().await;

        let (path, body) = requests
            .try_recv()
            .expect("shutdown should have exported the span");
        assert_eq!(path, "/v1/traces");
        // Protobuf carries strings as their UTF-8 bytes.
        assert!(
            body.windows(b"buffered".len())
                .any(|window| window == b"buffered"),
            "the exported batch should hold the span"
        );
    }
}
