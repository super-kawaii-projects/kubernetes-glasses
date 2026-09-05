use axum::{
    extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use shared::protocol::*;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

use crate::state::ClusterStateStore;

/// Start the HTTP + WebSocket server.
pub async fn serve(port: u16, state: ClusterStateStore) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/ws/frontend", get(frontend_ws_handler))
        .route("/ws/daemonset", get(daemonset_ws_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("API server listening on :{}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}

/// REST endpoint: get current cluster snapshot as JSON.
async fn snapshot_handler(State(state): State<ClusterStateStore>) -> impl IntoResponse {
    let snapshot = state.snapshot();
    axum::Json(snapshot)
}

// ─── Frontend WebSocket ──────────────────────────────────────────────────────
// Streams ClusterEvent to the frontend, accepts ClientCommand back.

async fn frontend_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ClusterStateStore>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_frontend_ws(socket, state))
}

async fn handle_frontend_ws(socket: WebSocket, state: ClusterStateStore) {
    let (mut sender, mut receiver) = socket.split();
    info!("Frontend client connected");

    // Send initial snapshot
    let snapshot = state.snapshot();
    let event = ClusterEvent::Snapshot(snapshot);
    if let Ok(msg) = serde_json::to_string(&event) {
        let _ = sender.send(Message::Text(msg.into())).await;
    }

    // Subscribe to real-time updates
    let mut rx = state.subscribe();

    // Spawn task to forward broadcast events to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(msg) = serde_json::to_string(&event) {
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming commands from the frontend
    let state_clone = state.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                    handle_client_command(cmd, &state_clone).await;
                }
            }
        }
    });

    // Wait for either task to finish (client disconnected)
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    info!("Frontend client disconnected");
}

async fn handle_client_command(cmd: ClientCommand, state: &ClusterStateStore) {
    match cmd {
        ClientCommand::RequestSnapshot => {
            // Already sent on connect — could re-send if needed
        }
        ClientCommand::Subscribe { namespaces: _ } => {
            // TODO: namespace filtering for this client
        }
        ClientCommand::GetPodLogs { .. } => {
            // TODO: proxy log request to kubelet via daemonset
        }
        ClientCommand::GetEvents { kind, name, namespace } => {
            let _events = state.get_events_for(&kind, &name, namespace.as_deref());
            // TODO: send response back to this specific client
        }
    }
}

// ─── DaemonSet WebSocket ─────────────────────────────────────────────────────
// Receives NodeReport messages from each daemonset agent.

async fn daemonset_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<ClusterStateStore>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_daemonset_ws(socket, state))
}

async fn handle_daemonset_ws(socket: WebSocket, state: ClusterStateStore) {
    let (mut _sender, mut receiver) = socket.split();
    info!("DaemonSet agent connected");

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                match serde_json::from_str::<NodeReport>(&text) {
                    Ok(report) => state.apply_node_report(report),
                    Err(e) => warn!("Invalid daemonset message: {}", e),
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    info!("DaemonSet agent disconnected");
}
