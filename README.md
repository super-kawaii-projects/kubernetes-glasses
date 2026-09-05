# kubernetes-glasses

Real-time Kubernetes cluster visualization. See your cluster's nodes, pods,
services, and topology at a glance.

Built in Rust with a [Leptos](https://leptos.dev) frontend, a controller
service that talks to the Kubernetes API, and an optional in-cluster
DaemonSet for node-level data.

## Architecture

```
kubernetes-glasses/
├── controller/     # Talks to the K8s API, serves cluster state (port 8080)
├── frontend/       # Leptos web UI (port 3000)
├── daemonset/      # Optional in-cluster agent for node-level metrics
├── shared/         # Shared types across components
├── deploy/         # Kubernetes deployment manifests
├── Dockerfile
└── docker-compose.yml
```

## Prerequisites

- Rust toolchain (see `rust-toolchain.toml`)
- A kubeconfig with access to the cluster you want to visualize
- Docker (for containerized runs)

## Development

```bash
docker compose up --build
```

- Controller API: http://localhost:8080
- Frontend UI: http://localhost:3000

The controller mounts your `~/.kube` config read-only to reach the cluster.

## In-cluster deployment

The DaemonSet runs inside the cluster (not via docker-compose). Apply the
manifests in `deploy/`:

```bash
kubectl apply -f deploy/
```

## License

Copyright (c) 2026 Stillwater Strategic Solutions LLC. All rights reserved.

Source-available software, not open source. Free for personal, non-commercial
evaluation. **Commercial and production use requires a paid license.** See
[LICENSE](LICENSE) or contact michaelisaacs121092@gmail.com for commercial
licensing.
