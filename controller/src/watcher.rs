use anyhow::Result;
use chrono::Utc;
use futures_util::{StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment as K8sDeployment;
use k8s_openapi::api::core::v1::{
    Namespace as K8sNamespace, Node as K8sNode, Service as K8sService,
};
use kube::{
    api::Api,
    runtime::watcher::{self, Event},
    Client,
};
use shared::models::*;
use std::collections::{BTreeMap, HashMap};
use tracing::{info, warn};

use crate::state::ClusterStateStore;

/// Convert BTreeMap (used by k8s-openapi) to HashMap (used by our models).
fn btree_to_hash(bt: BTreeMap<String, String>) -> HashMap<String, String> {
    bt.into_iter().collect()
}

/// Run all Kubernetes API watchers concurrently.
pub async fn run_watchers(state: ClusterStateStore) -> Result<()> {
    let client = Client::try_default().await?;
    info!("Connected to Kubernetes API server");

    let state_nodes = state.clone();
    let state_services = state.clone();
    let state_deployments = state.clone();
    let state_namespaces = state.clone();

    let client_nodes = client.clone();
    let client_services = client.clone();
    let client_deployments = client.clone();
    let client_namespaces = client.clone();

    tokio::try_join!(
        tokio::spawn(watch_nodes(client_nodes, state_nodes)),
        tokio::spawn(watch_services(client_services, state_services)),
        tokio::spawn(watch_deployments(client_deployments, state_deployments)),
        tokio::spawn(watch_namespaces(client_namespaces, state_namespaces)),
    )?;

    Ok(())
}

// ─── Node Watcher ────────────────────────────────────────────────────────────

async fn watch_nodes(client: Client, state: ClusterStateStore) {
    let api: Api<K8sNode> = Api::all(client);
    let watcher_config = watcher::Config::default();
    let mut stream = watcher::watcher(api, watcher_config).boxed();

    info!("Watching nodes...");
    while let Some(event) = stream.try_next().await.unwrap_or(None) {
        match event {
            Event::Apply(node) => {
                if let Some(converted) = convert_node(&node) {
                    state.upsert_node(converted);
                }
            }
            Event::Delete(node) => {
                let name = node.metadata.name.unwrap_or_default();
                state.remove_node(&name);
            }
            _ => {}
        }
    }
}

fn convert_node(k8s_node: &K8sNode) -> Option<Node> {
    let metadata = &k8s_node.metadata;
    let status = k8s_node.status.as_ref()?;
    let spec = k8s_node.spec.as_ref();

    let name = metadata.name.clone().unwrap_or_default();
    let uid = metadata.uid.clone().unwrap_or_default();
    let labels = btree_to_hash(metadata.labels.clone().unwrap_or_default());
    let annotations = btree_to_hash(metadata.annotations.clone().unwrap_or_default());

    // Determine node status from conditions
    let node_status = status.conditions.as_ref()
        .and_then(|conds| conds.iter().find(|c| c.type_ == "Ready"))
        .map(|c| {
            if c.status == "True" {
                if spec.and_then(|s| s.unschedulable).unwrap_or(false) {
                    NodeStatus::SchedulingDisabled
                } else {
                    NodeStatus::Ready
                }
            } else {
                NodeStatus::NotReady
            }
        })
        .unwrap_or(NodeStatus::Unknown);

    // Extract roles from labels
    let roles: Vec<String> = labels.iter()
        .filter(|(k, _)| k.starts_with("node-role.kubernetes.io/"))
        .map(|(k, _)| k.trim_start_matches("node-role.kubernetes.io/").to_string())
        .collect();

    let node_info = status.node_info.as_ref();
    let kubelet_version = node_info.map(|i| i.kubelet_version.clone()).unwrap_or_default();
    let os_image = node_info.map(|i| i.os_image.clone()).unwrap_or_default();
    let architecture = node_info.map(|i| i.architecture.clone()).unwrap_or_default();

    let internal_ip = status.addresses.as_ref()
        .and_then(|addrs| addrs.iter().find(|a| a.type_ == "InternalIP"))
        .map(|a| a.address.clone())
        .unwrap_or_default();

    let conditions = status.conditions.as_ref()
        .map(|conds| conds.iter().map(|c| NodeCondition {
            condition_type: c.type_.clone(),
            status: c.status.clone(),
            message: c.message.clone().unwrap_or_default(),
            last_transition: c.last_transition_time.as_ref()
                .map(|t| t.0)
                .unwrap_or_else(Utc::now),
        }).collect())
        .unwrap_or_default();

    let created_at = metadata.creation_timestamp.as_ref()
        .map(|t| t.0)
        .unwrap_or_else(Utc::now);

    Some(Node {
        name,
        uid,
        status: node_status,
        roles,
        labels,
        annotations,
        capacity: ResourceQuantity::default(),
        allocatable: ResourceQuantity::default(),
        usage: ResourceUsage::default(),
        conditions,
        pod_count: 0,
        kubelet_version,
        os_image,
        architecture,
        internal_ip,
        created_at,
    })
}

// ─── Service Watcher ─────────────────────────────────────────────────────────

async fn watch_services(client: Client, state: ClusterStateStore) {
    let api: Api<K8sService> = Api::all(client);
    let watcher_config = watcher::Config::default();
    let mut stream = watcher::watcher(api, watcher_config).boxed();

    info!("Watching services...");
    while let Some(event) = stream.try_next().await.unwrap_or(None) {
        match event {
            Event::Apply(svc) => {
                if let Some(converted) = convert_service(&svc) {
                    state.upsert_service(converted);
                }
            }
            Event::Delete(svc) => {
                let ns = svc.metadata.namespace.unwrap_or_else(|| "default".into());
                let name = svc.metadata.name.unwrap_or_default();
                state.remove_service(&ns, &name);
            }
            _ => {}
        }
    }
}

fn convert_service(k8s_svc: &K8sService) -> Option<Service> {
    let metadata = &k8s_svc.metadata;
    let spec = k8s_svc.spec.as_ref()?;

    let name = metadata.name.clone().unwrap_or_default();
    let namespace = metadata.namespace.clone().unwrap_or_else(|| "default".into());
    let uid = metadata.uid.clone().unwrap_or_default();
    let labels = btree_to_hash(metadata.labels.clone().unwrap_or_default());
    let selector = btree_to_hash(spec.selector.clone().unwrap_or_default());

    let service_type = match spec.type_.as_deref() {
        Some("NodePort") => ServiceType::NodePort,
        Some("LoadBalancer") => ServiceType::LoadBalancer,
        Some("ExternalName") => ServiceType::ExternalName,
        _ => ServiceType::ClusterIP,
    };

    let cluster_ip = spec.cluster_ip.clone();
    let external_ip = spec.external_ips.as_ref()
        .and_then(|ips| ips.first())
        .cloned();

    let ports = spec.ports.as_ref()
        .map(|ports| ports.iter().map(|p| ServicePort {
            name: p.name.clone(),
            port: p.port as u16,
            target_port: p.target_port.as_ref()
                .and_then(|tp| match tp {
                    k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => Some(*i as u16),
                    _ => None,
                })
                .unwrap_or(p.port as u16),
            node_port: p.node_port.map(|np| np as u16),
            protocol: p.protocol.clone().unwrap_or_else(|| "TCP".into()),
        }).collect())
        .unwrap_or_default();

    let created_at = metadata.creation_timestamp.as_ref()
        .map(|t| t.0)
        .unwrap_or_else(Utc::now);

    Some(Service {
        name,
        namespace,
        uid,
        service_type,
        cluster_ip,
        external_ip,
        ports,
        selector,
        labels,
        endpoint_count: 0,
        created_at,
    })
}

// ─── Deployment Watcher ──────────────────────────────────────────────────────

async fn watch_deployments(client: Client, state: ClusterStateStore) {
    let api: Api<K8sDeployment> = Api::all(client);
    let watcher_config = watcher::Config::default();
    let mut stream = watcher::watcher(api, watcher_config).boxed();

    info!("Watching deployments...");
    while let Some(event) = stream.try_next().await.unwrap_or(None) {
        match event {
            Event::Apply(deploy) => {
                if let Some(converted) = convert_deployment(&deploy) {
                    state.upsert_deployment(converted);
                }
            }
            Event::Delete(deploy) => {
                let ns = deploy.metadata.namespace.unwrap_or_else(|| "default".into());
                let name = deploy.metadata.name.unwrap_or_default();
                state.remove_deployment(&ns, &name);
            }
            _ => {}
        }
    }
}

fn convert_deployment(k8s_deploy: &K8sDeployment) -> Option<Deployment> {
    let metadata = &k8s_deploy.metadata;
    let spec = k8s_deploy.spec.as_ref()?;
    let status = k8s_deploy.status.as_ref();

    let name = metadata.name.clone().unwrap_or_default();
    let namespace = metadata.namespace.clone().unwrap_or_else(|| "default".into());
    let uid = metadata.uid.clone().unwrap_or_default();
    let labels = btree_to_hash(metadata.labels.clone().unwrap_or_default());

    let selector = btree_to_hash(spec.selector.match_labels.clone().unwrap_or_default());

    let replicas = spec.replicas.unwrap_or(1) as u32;
    let ready_replicas = status.and_then(|s| s.ready_replicas).unwrap_or(0) as u32;
    let available_replicas = status.and_then(|s| s.available_replicas).unwrap_or(0) as u32;
    let updated_replicas = status.and_then(|s| s.updated_replicas).unwrap_or(0) as u32;

    let strategy = spec.strategy.as_ref()
        .and_then(|s| match s.type_.as_deref() {
            Some("Recreate") => Some(DeploymentStrategy::Recreate),
            _ => {
                let ru = s.rolling_update.as_ref();
                Some(DeploymentStrategy::RollingUpdate {
                    max_surge: ru.and_then(|r| r.max_surge.as_ref())
                        .map(|v| format!("{:?}", v))
                        .unwrap_or_else(|| "25%".into()),
                    max_unavailable: ru.and_then(|r| r.max_unavailable.as_ref())
                        .map(|v| format!("{:?}", v))
                        .unwrap_or_else(|| "25%".into()),
                })
            }
        })
        .unwrap_or(DeploymentStrategy::RollingUpdate {
            max_surge: "25%".into(),
            max_unavailable: "25%".into(),
        });

    let conditions = status
        .and_then(|s| s.conditions.as_ref())
        .map(|conds| conds.iter().map(|c| DeploymentCondition {
            condition_type: c.type_.clone(),
            status: c.status.clone(),
            reason: c.reason.clone().unwrap_or_default(),
            message: c.message.clone().unwrap_or_default(),
            last_transition: c.last_transition_time.as_ref()
                .map(|t| t.0)
                .unwrap_or_else(Utc::now),
        }).collect())
        .unwrap_or_default();

    let created_at = metadata.creation_timestamp.as_ref()
        .map(|t| t.0)
        .unwrap_or_else(Utc::now);

    Some(Deployment {
        name,
        namespace,
        uid,
        replicas,
        ready_replicas,
        available_replicas,
        updated_replicas,
        strategy,
        labels,
        selector,
        conditions,
        created_at,
    })
}

// ─── Namespace Watcher ───────────────────────────────────────────────────────

async fn watch_namespaces(client: Client, state: ClusterStateStore) {
    let api: Api<K8sNamespace> = Api::all(client);
    let watcher_config = watcher::Config::default();
    let mut stream = watcher::watcher(api, watcher_config).boxed();

    info!("Watching namespaces...");
    while let Some(event) = stream.try_next().await.unwrap_or(None) {
        match event {
            Event::Apply(ns) => {
                if let Some(converted) = convert_namespace(&ns) {
                    state.upsert_namespace(converted);
                }
            }
            Event::Delete(ns) => {
                let name = ns.metadata.name.unwrap_or_default();
                state.remove_namespace(&name);
            }
            _ => {}
        }
    }
}

fn convert_namespace(k8s_ns: &K8sNamespace) -> Option<Namespace> {
    let metadata = &k8s_ns.metadata;
    let status = k8s_ns.status.as_ref();

    let name = metadata.name.clone().unwrap_or_default();
    let uid = metadata.uid.clone().unwrap_or_default();
    let labels = btree_to_hash(metadata.labels.clone().unwrap_or_default());

    let ns_status = status
        .and_then(|s| s.phase.as_deref())
        .map(|p| match p {
            "Terminating" => NamespaceStatus::Terminating,
            _ => NamespaceStatus::Active,
        })
        .unwrap_or(NamespaceStatus::Active);

    let created_at = metadata.creation_timestamp.as_ref()
        .map(|t| t.0)
        .unwrap_or_else(Utc::now);

    Some(Namespace {
        name,
        uid,
        status: ns_status,
        labels,
        created_at,
    })
}
