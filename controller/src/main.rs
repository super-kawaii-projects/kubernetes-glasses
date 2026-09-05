mod api;
mod state;
mod watcher;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("controller=info".parse()?))
        .init();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".into())
        .parse()
        .unwrap_or(8080);

    info!("kubernetes-glasses controller starting on :{}", port);

    // Initialize shared cluster state
    let cluster_state = state::ClusterStateStore::new();

    // Start Kubernetes API watchers (Services, Deployments, Namespaces, Events)
    let watcher_state = cluster_state.clone();
    tokio::spawn(async move {
        if let Err(e) = watcher::run_watchers(watcher_state).await {
            tracing::error!("K8s watcher failed: {}", e);
        }
    });

    // Start the HTTP/WebSocket API server
    api::serve(port, cluster_state).await?;

    Ok(())
}
