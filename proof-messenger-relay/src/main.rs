use axum::{
    routing::post,
    Router,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Message {
    msg_type: String,
    context: String,
    proof: String,
    pubkey: String,
    body: String,
}

async fn relay_message(Json(msg): Json<Message>) -> &'static str {
    // For demo: always "verify" and relay
    println!("Received: {:?}", msg);
    "Message relayed"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/relay", post(relay_message));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("🚀 Relay server starting...");
    println!("📡 Listening on 0.0.0.0:8080");
    println!("✅ Server ready to accept connections");
    axum::serve(listener, app).await.unwrap();
}