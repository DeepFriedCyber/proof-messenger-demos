use criterion::{black_box, criterion_group, criterion_main, Criterion};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use proof_messenger_relay::database::Database;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_test_app() -> Router {
    let db = Database::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();
    let db = Arc::new(db);

    // Create a router with just the health endpoint for benchmarking
    axum::Router::new()
        .route("/health", axum::routing::get(health_handler))
        .with_state(db)
}

// Simplified health handler for benchmarking
async fn health_handler(
    axum::extract::State(db): axum::extract::State<Arc<Database>>,
) -> impl axum::response::IntoResponse {
    match db.health_check().await {
        Ok(_) => {
            (StatusCode::OK, axum::Json(serde_json::json!({
                "status": "healthy",
                "database": "connected",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        },
        Err(e) => {
            (StatusCode::SERVICE_UNAVAILABLE, axum::Json(serde_json::json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string()
            })))
        }
    }
}

fn health_check_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Setup the test app
    let app = rt.block_on(async { setup_test_app().await });
    
    c.bench_function("health_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                let response = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("GET")
                            .uri("/health")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                
                black_box(response)
            })
        })
    });
}

criterion_group!(benches, health_check_benchmark);
criterion_main!(benches);