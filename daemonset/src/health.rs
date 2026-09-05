use axum::{routing::get, Router};
use tracing::info;

/// Serves /healthz and /readyz for Kubernetes probes.
pub async fn serve(port: u16) {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind health server");

    info!("Health server listening on :{}", port);

    axum::serve(listener, app).await.expect("Health server crashed");
}

async fn healthz() -> &'static str {
    "ok"
}

async fn readyz() -> &'static str {
    // TODO: check if we've successfully reported to controller recently
    "ok"
}
