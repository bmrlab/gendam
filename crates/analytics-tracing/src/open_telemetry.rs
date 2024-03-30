use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::{
    trace::TracerProvider,
    runtime,
    Resource,
    trace as sdktrace,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_opentelemetry::OpenTelemetryLayer;


pub(crate) fn init_otel_layer<S>() -> OpenTelemetryLayer<S, sdktrace::Tracer>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>
{
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let mut tracer = provider.tracer("cc.musedam.local");

    if let Ok(endpoint) = std::env::var("OPENTELEMETRY_OTLP_GRPC_ENDPOINT") {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint); // TODO, 需要改成线上的 gRPC 固定地址

        let otlp_config = sdktrace::config().with_resource(
            Resource::new(vec![
                opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    "cc.musedam.local",
                )
            ])
        );

        tracer = match opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(otlp_exporter)
            .with_trace_config(otlp_config)
            .install_batch(runtime::Tokio)
        {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to create OpenTelemetry tracer with otlp exporter: {}. Fallback to stdout", e);
                tracer
            }
        };
    }

    let telemetry = tracing_opentelemetry::layer::<S>()
        .with_tracer(tracer);

    telemetry
}
