use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::*;

// ─── DaemonSet → Controller Messages ────────────────────────────────────────
// Each daemonset agent on a node sends these to the controller.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeReport {
    /// Full snapshot of all pods/resources on this node
    FullSync(NodeSnapshot),
    /// Incremental update — a pod changed
    PodUpdate(PodDelta),
    /// Resource usage heartbeat
    Heartbeat(NodeHeartbeat),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSnapshot {
    pub node_name: String,
    pub pods: Vec<Pod>,
    pub node_usage: ResourceUsage,
    pub kubelet_healthy: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDelta {
    pub node_name: String,
    pub action: DeltaAction,
    pub pod: Pod,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeltaAction {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHeartbeat {
    pub node_name: String,
    pub cpu_millis: u64,
    pub memory_bytes: u64,
    pub pod_count: u32,
    pub kubelet_healthy: bool,
    pub timestamp: DateTime<Utc>,
}

// ─── Controller → Frontend Messages (WebSocket) ─────────────────────────────
// Streamed to the frontend for live UI updates.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterEvent {
    /// Initial full cluster state on connect
    Snapshot(ClusterState),
    /// A node's status/resources changed
    NodeUpdate(Node),
    /// A pod was added/modified/deleted
    PodEvent { action: DeltaAction, pod: Pod },
    /// A service changed
    ServiceUpdate(Service),
    /// A deployment changed
    DeploymentUpdate(Deployment),
    /// A Kubernetes event occurred
    Event(KubeEvent),
    /// A node went offline (daemonset stopped reporting)
    NodeOffline { node_name: String, last_seen: DateTime<Utc> },
    /// Error from controller
    Error { message: String },
}

// ─── Frontend → Controller Messages (WebSocket) ─────────────────────────────
// Commands the frontend can send to the controller.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCommand {
    /// Request full cluster snapshot
    RequestSnapshot,
    /// Subscribe to a specific namespace (empty = all)
    Subscribe { namespaces: Vec<String> },
    /// Request pod logs
    GetPodLogs {
        namespace: String,
        pod_name: String,
        container: Option<String>,
        tail_lines: u32,
    },
    /// Request events for a specific object
    GetEvents {
        kind: String,
        name: String,
        namespace: Option<String>,
    },
}

// ─── Responses ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientResponse {
    PodLogs {
        pod_name: String,
        container: String,
        lines: Vec<String>,
    },
    Events {
        events: Vec<KubeEvent>,
    },
    Error {
        message: String,
    },
}
