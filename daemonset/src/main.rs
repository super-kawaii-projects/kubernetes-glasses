mod collector;
mod reporter;
mod health;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("daemonset=info".parse()?))
        .init();

    let node_name = std::env::var("NODE_NAME")
        .unwrap_or_else(|_| hostname().unwrap_or_else(|| "unknown".into()));
    let controller_url = std::env::var("CONTROLLER_URL")
        .unwrap_or_else(|_| "ws://controller:8080/ws/daemonset".into());
    let kubelet_port: u16 = std::env::var("KUBELET_PORT")
        .unwrap_or_else(|_| "10250".into())
        .parse()
        .unwrap_or(10250);
    let health_port: u16 = std::env::var("HEALTH_PORT")
        .unwrap_or_else(|_| "9090".into())
        .parse()
        .unwrap_or(9090);
    let sync_interval_secs: u64 = std::env::var("SYNC_INTERVAL_SECS")
        .unwrap_or_else(|_| "15".into())
        .parse()
        .unwrap_or(15);

    info!("kubernetes-glasses daemonset starting");
    info!("  node:       {}", node_name);
    info!("  controller: {}", controller_url);
    info!("  kubelet:    localhost:{}", kubelet_port);
    info!("  health:     :{}", health_port);
    info!("  sync every: {}s", sync_interval_secs);

    // Start health server in background
    let health_handle = tokio::spawn(health::serve(health_port));

    // Start the collector → reporter pipeline
    let collector = collector::KubeletCollector::new(&node_name, kubelet_port);
    let mut reporter = reporter::ControllerReporter::new(&controller_url, &node_name);

    // Main loop: collect from kubelet, report to controller
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(sync_interval_secs));
    let mut full_sync_counter: u64 = 0;

    loop {
        interval.tick().await;
        full_sync_counter += 1;

        // Every 4th tick, do a full sync. Otherwise, heartbeat.
        if full_sync_counter % 4 == 1 {
            match collector.collect_snapshot().await {
                Ok(snapshot) => {
                    if let Err(e) = reporter.send_full_sync(snapshot).await {
                        tracing::warn!("Failed to send full sync: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to collect snapshot: {}", e);
                    reporter.send_unhealthy_heartbeat().await.ok();
                }
            }
        } else {
            match collector.collect_heartbeat().await {
                Ok(heartbeat) => {
                    if let Err(e) = reporter.send_heartbeat(heartbeat).await {
                        tracing::warn!("Failed to send heartbeat: {}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to collect heartbeat: {}", e);
                }
            }
        }
    }

    // Unreachable, but for completeness
    #[allow(unreachable_code)]
    {
        health_handle.abort();
        Ok(())
    }
}

fn hostname() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|h| h.trim().to_string())
}
