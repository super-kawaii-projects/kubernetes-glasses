use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Cluster ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<Node>,
    pub pods: Vec<Pod>,
    pub services: Vec<Service>,
    pub deployments: Vec<Deployment>,
    pub namespaces: Vec<Namespace>,
    pub last_updated: DateTime<Utc>,
}

// ─── Node ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub uid: String,
    pub status: NodeStatus,
    pub roles: Vec<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub capacity: ResourceQuantity,
    pub allocatable: ResourceQuantity,
    pub usage: ResourceUsage,
    pub conditions: Vec<NodeCondition>,
    pub pod_count: u32,
    pub kubelet_version: String,
    pub os_image: String,
    pub architecture: String,
    pub internal_ip: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Ready,
    NotReady,
    Unknown,
    SchedulingDisabled,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Ready => write!(f, "Ready"),
            NodeStatus::NotReady => write!(f, "NotReady"),
            NodeStatus::Unknown => write!(f, "Unknown"),
            NodeStatus::SchedulingDisabled => write!(f, "SchedulingDisabled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
    pub message: String,
    pub last_transition: DateTime<Utc>,
}

// ─── Pod ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub node_name: String,
    pub status: PodStatus,
    pub phase: PodPhase,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub containers: Vec<Container>,
    pub ip: Option<String>,
    pub qos_class: QosClass,
    pub restart_count: u32,
    pub owner_kind: Option<String>,
    pub owner_name: Option<String>,
    pub usage: ResourceUsage,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PodPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl std::fmt::Display for PodPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PodPhase::Pending => write!(f, "Pending"),
            PodPhase::Running => write!(f, "Running"),
            PodPhase::Succeeded => write!(f, "Succeeded"),
            PodPhase::Failed => write!(f, "Failed"),
            PodPhase::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    pub ready: bool,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QosClass {
    Guaranteed,
    Burstable,
    BestEffort,
}

// ─── Container ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub state: ContainerState,
    pub ready: bool,
    pub restart_count: u32,
    pub requests: ResourceQuantity,
    pub limits: ResourceQuantity,
    pub usage: ResourceUsage,
    pub ports: Vec<ContainerPort>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerState {
    Running { started_at: DateTime<Utc> },
    Waiting { reason: String },
    Terminated { reason: String, exit_code: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub name: Option<String>,
    pub container_port: u16,
    pub protocol: String,
}

// ─── Service ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub service_type: ServiceType,
    pub cluster_ip: Option<String>,
    pub external_ip: Option<String>,
    pub ports: Vec<ServicePort>,
    pub selector: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub endpoint_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::ClusterIP => write!(f, "ClusterIP"),
            ServiceType::NodePort => write!(f, "NodePort"),
            ServiceType::LoadBalancer => write!(f, "LoadBalancer"),
            ServiceType::ExternalName => write!(f, "ExternalName"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub port: u16,
    pub target_port: u16,
    pub node_port: Option<u16>,
    pub protocol: String,
}

// ─── Deployment ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub name: String,
    pub namespace: String,
    pub uid: String,
    pub replicas: u32,
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub updated_replicas: u32,
    pub strategy: DeploymentStrategy,
    pub labels: HashMap<String, String>,
    pub selector: HashMap<String, String>,
    pub conditions: Vec<DeploymentCondition>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStrategy {
    RollingUpdate { max_surge: String, max_unavailable: String },
    Recreate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCondition {
    pub condition_type: String,
    pub status: String,
    pub reason: String,
    pub message: String,
    pub last_transition: DateTime<Utc>,
}

// ─── Namespace ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub uid: String,
    pub status: NamespaceStatus,
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NamespaceStatus {
    Active,
    Terminating,
}

// ─── Resources ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceQuantity {
    /// CPU in millicores (e.g., 1000 = 1 CPU)
    pub cpu_millis: u64,
    /// Memory in bytes
    pub memory_bytes: u64,
    /// Ephemeral storage in bytes
    pub storage_bytes: u64,
    /// Number of pods (for node capacity)
    pub pods: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    /// CPU in millicores (current usage)
    pub cpu_millis: u64,
    /// Memory in bytes (current usage)
    pub memory_bytes: u64,
    /// Timestamp of this measurement
    pub timestamp: Option<DateTime<Utc>>,
}

// ─── Events ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubeEvent {
    pub uid: String,
    pub event_type: EventType,
    pub reason: String,
    pub message: String,
    pub involved_object: ObjectReference,
    pub count: u32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Normal,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectReference {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
}
