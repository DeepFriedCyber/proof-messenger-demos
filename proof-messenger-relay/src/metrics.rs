// proof-messenger-relay/src/metrics.rs
use once_cell::sync::Lazy;
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::Instant;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing;

// 1. Create a registry to hold all our metrics.
// Using Lazy from once_cell means this will be initialized exactly once.
pub static APP_REGISTRY: Lazy<Arc<Registry>> = Lazy::new(|| {
    let mut registry = Registry::default();
    
    // Register the metrics we define below.
    registry.register(
        "http_requests_total",
        "Total number of HTTP requests handled",
        HTTP_REQUESTS_TOTAL.clone(),
    );
    
    registry.register(
        "http_requests_latency_seconds",
        "HTTP request latency in seconds",
        HTTP_REQUESTS_LATENCY_SECONDS.clone(),
    );
    
    Arc::new(registry)
});

// 2. Define our metrics.
// A counter for total requests.
pub static HTTP_REQUESTS_TOTAL: Lazy<Counter> = Lazy::new(Counter::default);

// A histogram to track request latencies.
pub static HTTP_REQUESTS_LATENCY_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    // Start at 100 microseconds, multiply by 2 for each bucket, 12 buckets total.
    let buckets = exponential_buckets(0.0001, 2.0, 12);
    Histogram::new(buckets)
});

// 3. A handler function that we'll use for our /metrics endpoint.
pub async fn metrics_handler() -> (
    axum::http::StatusCode,
    axum::http::HeaderMap,
    String,
) {
    tracing::info!("Metrics endpoint called");
    
    let mut buffer = String::new();
    encode(&mut buffer, &APP_REGISTRY.as_ref()).unwrap();
    
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/openmetrics-text; version=1.0.0; charset=utf-8"
            .parse()
            .unwrap(),
    );
    
    tracing::info!("Metrics response prepared, buffer length: {}", buffer.len());
    (axum::http::StatusCode::OK, headers, buffer)
}

// 4. Enhanced middleware to automatically track HTTP requests with labels
pub async fn metrics_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    
    // Record the path for labeling metrics
    let path = request.uri().path().to_string();
    let method = request.method().clone();
    
    // Increment the request counter
    HTTP_REQUESTS_TOTAL.inc();
    
    // Process the request
    let response = next.run(request).await;
    
    // Record the latency and status
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    
    HTTP_REQUESTS_LATENCY_SECONDS.observe(latency);
    
    // Log the request details for debugging
    tracing::debug!(
        method = %method,
        path = %path,
        status = %status,
        latency_ms = latency * 1000.0,
        "HTTP request processed"
    );
    
    response
}