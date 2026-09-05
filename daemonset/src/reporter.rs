use anyhow::{Context, Result};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use shared::protocol::*;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn};

/// Reports collected data from this node to the central controller via WebSocket.
pub struct ControllerReporter {
    controller_url: String,
    node_name: String,
    /// Holds the active WebSocket connection (lazily connected)
    connection: Option<WebSocketConnection>,
}

type WebSocketConnection = (
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    >,
);

impl ControllerReporter {
    pub fn new(controller_url: &str, node_name: &str) -> Self {
        Self {
            controller_url: controller_url.to_string(),
            node_name: node_name.to_string(),
            connection: None,
        }
    }

    /// Send a full node snapshot to the controller.
    pub async fn send_full_sync(&mut self, snapshot: NodeSnapshot) -> Result<()> {
        let report = NodeReport::FullSync(snapshot);
        self.send_report(report).await
    }

    /// Send a heartbeat to the controller.
    pub async fn send_heartbeat(&mut self, heartbeat: NodeHeartbeat) -> Result<()> {
        let report = NodeReport::Heartbeat(heartbeat);
        self.send_report(report).await
    }

    /// Send an unhealthy heartbeat when we can't reach kubelet.
    pub async fn send_unhealthy_heartbeat(&mut self) -> Result<()> {
        let heartbeat = NodeHeartbeat {
            node_name: self.node_name.clone(),
            cpu_millis: 0,
            memory_bytes: 0,
            pod_count: 0,
            kubelet_healthy: false,
            timestamp: Utc::now(),
        };
        self.send_report(NodeReport::Heartbeat(heartbeat)).await
    }

    /// Send a pod delta (add/modify/delete) to the controller.
    pub async fn send_pod_update(&mut self, delta: PodDelta) -> Result<()> {
        let report = NodeReport::PodUpdate(delta);
        self.send_report(report).await
    }

    // ─── Internal ────────────────────────────────────────────────────────────

    async fn send_report(&mut self, report: NodeReport) -> Result<()> {
        let msg = serde_json::to_string(&report).context("Failed to serialize report")?;
        let sink = self.get_connection().await?;
        sink.send(Message::Text(msg.into())).await.context("Failed to send WebSocket message")?;
        debug!("Sent report to controller");
        Ok(())
    }

    async fn get_connection(
        &mut self,
    ) -> Result<
        &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
    > {
        if self.connection.is_none() {
            self.connect().await?;
        }

        match &mut self.connection {
            Some((sink, _)) => Ok(sink),
            None => anyhow::bail!("No connection available"),
        }
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to controller: {}", self.controller_url);

        let (ws_stream, _) = connect_async(&self.controller_url)
            .await
            .context("Failed to connect to controller WebSocket")?;

        let (sink, stream) = ws_stream.split();
        self.connection = Some((sink, stream));
        info!("Connected to controller");
        Ok(())
    }

    /// Reconnect if the connection was lost.
    #[allow(dead_code)]
    pub async fn reconnect(&mut self) -> Result<()> {
        warn!("Reconnecting to controller...");
        self.connection = None;
        self.connect().await
    }
}
