use anyhow::{Context, Result};
use chrono::Utc;
use shared::models::*;
use shared::protocol::*;
use tracing::debug;

/// Scrapes the local kubelet API for pod/resource information.
/// Kubelet exposes:
///   - /pods          → all pods on this node
///   - /stats/summary → node + pod resource usage
///   - /healthz       → kubelet health
pub struct KubeletCollector {
    node_name: String,
    client: reqwest::Client,
    base_url: String,
}

impl KubeletCollector {
    pub fn new(node_name: &str, kubelet_port: u16) -> Self {
        // Kubelet uses self-signed certs — we need to accept them.
        // In production, use the service account token at
        // /var/run/secrets/kubernetes.io/serviceaccount/token
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            node_name: node_name.to_string(),
            client,
            base_url: format!("https://localhost:{}", kubelet_port),
        }
    }

    /// Collect a full snapshot of all pods + resource usage on this node.
    pub async fn collect_snapshot(&self) -> Result<NodeSnapshot> {
        debug!("Collecting full snapshot from kubelet");

        let pods = self.get_pods().await.unwrap_or_default();
        let node_usage = self.get_node_usage().await.unwrap_or_default();
        let healthy = self.check_health().await;

        Ok(NodeSnapshot {
            node_name: self.node_name.clone(),
            pods,
            node_usage,
            kubelet_healthy: healthy,
            timestamp: Utc::now(),
        })
    }

    /// Collect a lightweight heartbeat with just resource numbers.
    pub async fn collect_heartbeat(&self) -> Result<NodeHeartbeat> {
        let usage = self.get_node_usage().await.unwrap_or_default();
        let pod_count = self.get_pod_count().await.unwrap_or(0);
        let healthy = self.check_health().await;

        Ok(NodeHeartbeat {
            node_name: self.node_name.clone(),
            cpu_millis: usage.cpu_millis,
            memory_bytes: usage.memory_bytes,
            pod_count,
            kubelet_healthy: healthy,
            timestamp: Utc::now(),
        })
    }

    /// GET /healthz
    async fn check_health(&self) -> bool {
        let url = format!("{}/healthz", self.base_url);
        match self.client.get(&url).bearer_auth(self.token()).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// GET /pods — returns kubelet pod list
    async fn get_pods(&self) -> Result<Vec<Pod>> {
        let url = format!("{}/pods", self.base_url);
        let resp = self.client
            .get(&url)
            .bearer_auth(self.token())
            .send()
            .await
            .context("Failed to reach kubelet /pods")?;

        let body: serde_json::Value = resp.json().await.context("Failed to parse /pods response")?;
        let pods = parse_pod_list(&body, &self.node_name);
        Ok(pods)
    }

    /// GET /stats/summary — node-level resource usage
    async fn get_node_usage(&self) -> Result<ResourceUsage> {
        let url = format!("{}/stats/summary", self.base_url);
        let resp = self.client
            .get(&url)
            .bearer_auth(self.token())
            .send()
            .await
            .context("Failed to reach kubelet /stats/summary")?;

        let body: serde_json::Value = resp.json().await.context("Failed to parse /stats/summary")?;
        Ok(parse_node_stats(&body))
    }

    async fn get_pod_count(&self) -> Result<u32> {
        let url = format!("{}/pods", self.base_url);
        let resp = self.client
            .get(&url)
            .bearer_auth(self.token())
            .send()
            .await?;
        let body: serde_json::Value = resp.json().await?;
        let count = body.get("items")
            .and_then(|i| i.as_array())
            .map(|a| a.len() as u32)
            .unwrap_or(0);
        Ok(count)
    }

    /// Read the ServiceAccount token mounted by Kubernetes.
    fn token(&self) -> String {
        std::fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")
            .unwrap_or_default()
    }
}

// ─── Parsers ─────────────────────────────────────────────────────────────────
// Parse kubelet JSON responses into our shared models.

fn parse_pod_list(body: &serde_json::Value, node_name: &str) -> Vec<Pod> {
    let items = match body.get("items").and_then(|i| i.as_array()) {
        Some(items) => items,
        None => return Vec::new(),
    };

    items.iter().filter_map(|item| parse_single_pod(item, node_name)).collect()
}

fn parse_single_pod(item: &serde_json::Value, node_name: &str) -> Option<Pod> {
    let metadata = item.get("metadata")?;
    let spec = item.get("spec")?;
    let status = item.get("status")?;

    let name = metadata.get("name")?.as_str()?.to_string();
    let namespace = metadata.get("namespace")?.as_str().unwrap_or("default").to_string();
    let uid = metadata.get("uid")?.as_str().unwrap_or("").to_string();

    let phase_str = status.get("phase").and_then(|p| p.as_str()).unwrap_or("Unknown");
    let phase = match phase_str {
        "Running" => PodPhase::Running,
        "Pending" => PodPhase::Pending,
        "Succeeded" => PodPhase::Succeeded,
        "Failed" => PodPhase::Failed,
        _ => PodPhase::Unknown,
    };

    let pod_ip = status.get("podIP").and_then(|ip| ip.as_str()).map(|s| s.to_string());

    let qos = spec.get("qosClass")
        .or_else(|| status.get("qosClass"))
        .and_then(|q| q.as_str())
        .unwrap_or("BestEffort");
    let qos_class = match qos {
        "Guaranteed" => QosClass::Guaranteed,
        "Burstable" => QosClass::Burstable,
        _ => QosClass::BestEffort,
    };

    let labels = parse_string_map(metadata.get("labels"));
    let annotations = parse_string_map(metadata.get("annotations"));

    // Owner reference
    let owner = metadata.get("ownerReferences")
        .and_then(|o| o.as_array())
        .and_then(|a| a.first());
    let owner_kind = owner.and_then(|o| o.get("kind")).and_then(|k| k.as_str()).map(|s| s.to_string());
    let owner_name = owner.and_then(|o| o.get("name")).and_then(|n| n.as_str()).map(|s| s.to_string());

    // Containers
    let containers = parse_containers(spec, status);

    let restart_count: u32 = containers.iter().map(|c| c.restart_count).sum();

    Some(Pod {
        name,
        namespace,
        uid,
        node_name: node_name.to_string(),
        status: PodStatus {
            ready: phase == PodPhase::Running,
            reason: status.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string()),
            message: status.get("message").and_then(|m| m.as_str()).map(|s| s.to_string()),
        },
        phase,
        labels,
        annotations,
        containers,
        ip: pod_ip,
        qos_class,
        restart_count,
        owner_kind,
        owner_name,
        usage: ResourceUsage::default(),
        created_at: parse_timestamp(metadata.get("creationTimestamp")),
    })
}

fn parse_containers(spec: &serde_json::Value, status: &serde_json::Value) -> Vec<Container> {
    let spec_containers = spec.get("containers").and_then(|c| c.as_array());
    let status_containers = status.get("containerStatuses").and_then(|c| c.as_array());

    let Some(specs) = spec_containers else { return Vec::new() };

    specs.iter().map(|cs| {
        let name = cs.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
        let image = cs.get("image").and_then(|i| i.as_str()).unwrap_or("").to_string();

        // Find matching container status
        let cs_status = status_containers
            .and_then(|statuses| statuses.iter().find(|s| {
                s.get("name").and_then(|n| n.as_str()) == Some(&name)
            }));

        let state = parse_container_state(cs_status);
        let ready = cs_status
            .and_then(|s| s.get("ready"))
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        let restart_count = cs_status
            .and_then(|s| s.get("restartCount"))
            .and_then(|r| r.as_u64())
            .unwrap_or(0) as u32;

        let ports = cs.get("ports")
            .and_then(|p| p.as_array())
            .map(|ports| ports.iter().filter_map(|p| {
                Some(ContainerPort {
                    name: p.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                    container_port: p.get("containerPort")?.as_u64()? as u16,
                    protocol: p.get("protocol").and_then(|pr| pr.as_str()).unwrap_or("TCP").to_string(),
                })
            }).collect())
            .unwrap_or_default();

        Container {
            name,
            image,
            state,
            ready,
            restart_count,
            requests: ResourceQuantity::default(),
            limits: ResourceQuantity::default(),
            usage: ResourceUsage::default(),
            ports,
        }
    }).collect()
}

fn parse_container_state(status: Option<&serde_json::Value>) -> ContainerState {
    let Some(status) = status else {
        return ContainerState::Waiting { reason: "Unknown".into() };
    };

    let state = match status.get("state") {
        Some(s) => s,
        None => return ContainerState::Waiting { reason: "Unknown".into() },
    };

    if let Some(running) = state.get("running") {
        let started = parse_timestamp(running.get("startedAt"));
        ContainerState::Running { started_at: started }
    } else if let Some(waiting) = state.get("waiting") {
        let reason = waiting.get("reason").and_then(|r| r.as_str()).unwrap_or("Unknown").to_string();
        ContainerState::Waiting { reason }
    } else if let Some(terminated) = state.get("terminated") {
        let reason = terminated.get("reason").and_then(|r| r.as_str()).unwrap_or("Unknown").to_string();
        let exit_code = terminated.get("exitCode").and_then(|e| e.as_i64()).unwrap_or(-1) as i32;
        ContainerState::Terminated { reason, exit_code }
    } else {
        ContainerState::Waiting { reason: "Unknown".into() }
    }
}

fn parse_node_stats(body: &serde_json::Value) -> ResourceUsage {
    let node = match body.get("node") {
        Some(n) => n,
        None => return ResourceUsage::default(),
    };

    let cpu_millis = node.get("cpu")
        .and_then(|c| c.get("usageNanoCores"))
        .and_then(|u| u.as_u64())
        .map(|nanos| nanos / 1_000_000) // nano → milli
        .unwrap_or(0);

    let memory_bytes = node.get("memory")
        .and_then(|m| m.get("usageBytes"))
        .and_then(|u| u.as_u64())
        .unwrap_or(0);

    ResourceUsage {
        cpu_millis,
        memory_bytes,
        timestamp: Some(Utc::now()),
    }
}

fn parse_string_map(val: Option<&serde_json::Value>) -> std::collections::HashMap<String, String> {
    val.and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_timestamp(val: Option<&serde_json::Value>) -> chrono::DateTime<Utc> {
    val.and_then(|v| v.as_str())
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now)
}
