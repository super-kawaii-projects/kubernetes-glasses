use chrono::{DateTime, Utc};
use dashmap::DashMap;
use shared::models::*;
use shared::protocol::*;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;
use uuid::Uuid;

/// Central cluster state store. Thread-safe, shared across all tasks.
#[derive(Clone)]
pub struct ClusterStateStore {
    inner: Arc<Inner>,
}

struct Inner {
    /// All known nodes (by node name)
    pub nodes: DashMap<String, Node>,
    /// All known pods (by "namespace/name")
    pub pods: DashMap<String, Pod>,
    /// All known services (by "namespace/name")
    pub services: DashMap<String, Service>,
    /// All known deployments (by "namespace/name")
    pub deployments: DashMap<String, Deployment>,
    /// All known namespaces
    pub namespaces: DashMap<String, Namespace>,
    /// Recent events (ring buffer — last 1000)
    pub events: DashMap<String, KubeEvent>,
    /// Broadcast channel for real-time updates to frontend clients
    pub event_tx: broadcast::Sender<ClusterEvent>,
    /// Cluster ID
    pub cluster_id: Uuid,
}

impl ClusterStateStore {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            inner: Arc::new(Inner {
                nodes: DashMap::new(),
                pods: DashMap::new(),
                services: DashMap::new(),
                deployments: DashMap::new(),
                namespaces: DashMap::new(),
                events: DashMap::new(),
                event_tx,
                cluster_id: Uuid::new_v4(),
            }),
        }
    }

    /// Subscribe to real-time cluster events (for frontend WebSocket clients).
    pub fn subscribe(&self) -> broadcast::Receiver<ClusterEvent> {
        self.inner.event_tx.subscribe()
    }

    /// Get a full snapshot of the current cluster state.
    pub fn snapshot(&self) -> ClusterState {
        ClusterState {
            id: self.inner.cluster_id,
            name: std::env::var("CLUSTER_NAME").unwrap_or_else(|_| "default".into()),
            nodes: self.inner.nodes.iter().map(|r| r.value().clone()).collect(),
            pods: self.inner.pods.iter().map(|r| r.value().clone()).collect(),
            services: self.inner.services.iter().map(|r| r.value().clone()).collect(),
            deployments: self.inner.deployments.iter().map(|r| r.value().clone()).collect(),
            namespaces: self.inner.namespaces.iter().map(|r| r.value().clone()).collect(),
            last_updated: Utc::now(),
        }
    }

    // ─── DaemonSet Reports ───────────────────────────────────────────────────

    /// Process a full sync report from a daemonset agent.
    pub fn apply_node_report(&self, report: NodeReport) {
        match report {
            NodeReport::FullSync(snapshot) => self.apply_full_sync(snapshot),
            NodeReport::PodUpdate(delta) => self.apply_pod_delta(delta),
            NodeReport::Heartbeat(heartbeat) => self.apply_heartbeat(heartbeat),
        }
    }

    fn apply_full_sync(&self, snapshot: NodeSnapshot) {
        info!("Full sync from node: {} ({} pods)", snapshot.node_name, snapshot.pods.len());

        // Remove old pods from this node, then insert new ones
        self.inner.pods.retain(|_, pod| pod.node_name != snapshot.node_name);

        for pod in &snapshot.pods {
            let key = format!("{}/{}", pod.namespace, pod.name);
            self.inner.pods.insert(key, pod.clone());
        }

        // Update node usage info
        if let Some(mut node) = self.inner.nodes.get_mut(&snapshot.node_name) {
            node.usage = snapshot.node_usage.clone();
            node.pod_count = snapshot.pods.len() as u32;
        }

        // Broadcast to frontend clients
        let _ = self.inner.event_tx.send(ClusterEvent::Snapshot(self.snapshot()));
    }

    fn apply_pod_delta(&self, delta: PodDelta) {
        let key = format!("{}/{}", delta.pod.namespace, delta.pod.name);

        match delta.action {
            DeltaAction::Added | DeltaAction::Modified => {
                self.inner.pods.insert(key, delta.pod.clone());
            }
            DeltaAction::Deleted => {
                self.inner.pods.remove(&key);
            }
        }

        let _ = self.inner.event_tx.send(ClusterEvent::PodEvent {
            action: delta.action,
            pod: delta.pod,
        });
    }

    fn apply_heartbeat(&self, heartbeat: NodeHeartbeat) {
        if let Some(mut node) = self.inner.nodes.get_mut(&heartbeat.node_name) {
            node.usage.cpu_millis = heartbeat.cpu_millis;
            node.usage.memory_bytes = heartbeat.memory_bytes;
            node.usage.timestamp = Some(heartbeat.timestamp);
            node.pod_count = heartbeat.pod_count;
        }

        // Check for unhealthy node
        if !heartbeat.kubelet_healthy {
            let _ = self.inner.event_tx.send(ClusterEvent::NodeOffline {
                node_name: heartbeat.node_name,
                last_seen: heartbeat.timestamp,
            });
        }
    }

    // ─── K8s API Watcher Updates ─────────────────────────────────────────────

    pub fn upsert_node(&self, node: Node) {
        let _ = self.inner.event_tx.send(ClusterEvent::NodeUpdate(node.clone()));
        self.inner.nodes.insert(node.name.clone(), node);
    }

    pub fn remove_node(&self, name: &str) {
        self.inner.nodes.remove(name);
        let _ = self.inner.event_tx.send(ClusterEvent::NodeOffline {
            node_name: name.to_string(),
            last_seen: Utc::now(),
        });
    }

    pub fn upsert_service(&self, service: Service) {
        let _ = self.inner.event_tx.send(ClusterEvent::ServiceUpdate(service.clone()));
        let key = format!("{}/{}", service.namespace, service.name);
        self.inner.services.insert(key, service);
    }

    pub fn remove_service(&self, namespace: &str, name: &str) {
        let key = format!("{}/{}", namespace, name);
        self.inner.services.remove(&key);
    }

    pub fn upsert_deployment(&self, deployment: Deployment) {
        let _ = self.inner.event_tx.send(ClusterEvent::DeploymentUpdate(deployment.clone()));
        let key = format!("{}/{}", deployment.namespace, deployment.name);
        self.inner.deployments.insert(key, deployment);
    }

    pub fn remove_deployment(&self, namespace: &str, name: &str) {
        let key = format!("{}/{}", namespace, name);
        self.inner.deployments.remove(&key);
    }

    pub fn upsert_namespace(&self, ns: Namespace) {
        self.inner.namespaces.insert(ns.name.clone(), ns);
    }

    pub fn remove_namespace(&self, name: &str) {
        self.inner.namespaces.remove(name);
    }

    pub fn add_event(&self, event: KubeEvent) {
        let _ = self.inner.event_tx.send(ClusterEvent::Event(event.clone()));
        self.inner.events.insert(event.uid.clone(), event);
        // Trim to last 1000 events
        while self.inner.events.len() > 1000 {
            if let Some(oldest) = self.inner.events.iter().next() {
                let key = oldest.key().clone();
                drop(oldest);
                self.inner.events.remove(&key);
            }
        }
    }

    pub fn get_events_for(&self, kind: &str, name: &str, namespace: Option<&str>) -> Vec<KubeEvent> {
        self.inner.events.iter()
            .filter(|e| {
                let obj = &e.involved_object;
                obj.kind == kind && obj.name == name
                    && (namespace.is_none() || obj.namespace.as_deref() == namespace)
            })
            .map(|e| e.value().clone())
            .collect()
    }
}
