# kubernetes-glasses

**See your Kubernetes cluster clearly — live nodes, pods, services, and topology in one clean web UI.**

kubernetes-glasses runs inside your cluster, watches the Kubernetes API in real time, and renders what's actually happening: which pods are where, how services connect, node health, and a live topology view of the whole thing. Install it with one Helm command and port-forward to the dashboard.

Built in Rust — a lightweight controller, an optional per-node DaemonSet, and a [Leptos](https://leptos.dev) web frontend.

---

## Features

- **Cluster overview** — namespaces, workloads, and health at a glance
- **Nodes** — capacity, conditions, and per-node status
- **Pods** — live status across namespaces, filtered and searchable
- **Services** — how traffic routes, endpoints, and selectors
- **Live topology** — a visual map of how everything connects, updated in real time
- **Low footprint** — the controller requests just 100m CPU / 128Mi memory; the node agent even less

---

## Install with Helm

```bash
# from a checkout of this repo
helm install kubernetes-glasses ./charts/kubernetes-glasses \
  --namespace kubernetes-glasses --create-namespace
```

Then open the dashboard:

```bash
kubectl port-forward -n kubernetes-glasses \
  svc/kubernetes-glasses-frontend 3000:3000
```

Browse to **http://localhost:3000**.

### Common Helm options

```bash
# Pin a specific image tag
helm install kubernetes-glasses ./charts/kubernetes-glasses \
  --namespace kubernetes-glasses --create-namespace \
  --set image.tag=0.1.0

# Skip the per-node DaemonSet (controller + UI only)
helm install kubernetes-glasses ./charts/kubernetes-glasses \
  --namespace kubernetes-glasses --create-namespace \
  --set daemonset.enabled=false

# Expose the UI via Ingress instead of port-forward
helm install kubernetes-glasses ./charts/kubernetes-glasses \
  --namespace kubernetes-glasses --create-namespace \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=k8s-glasses.example.com \
  --set ingress.className=nginx
```

### Key values

| Value | Description | Default |
|-------|-------------|---------|
| `image.repository` | Container image | `ghcr.io/super-kawaii-projects/kubernetes-glasses` |
| `image.tag` | Image tag (defaults to chart appVersion) | `""` |
| `controller.enabled` | Deploy the API controller | `true` |
| `frontend.enabled` | Deploy the web UI | `true` |
| `daemonset.enabled` | Deploy the per-node agent | `true` |
| `daemonset.tolerateAllTaints` | Run on every node (incl. control-plane) | `true` |
| `frontend.service.port` | UI port | `3000` |
| `controller.service.port` | Controller API port | `8080` |
| `ingress.enabled` | Expose the UI via Ingress | `false` |
| `rbac.create` | Create the required ClusterRoles/Bindings | `true` |
| `serviceAccount.create` | Create the ServiceAccounts | `true` |

See [`charts/kubernetes-glasses/values.yaml`](charts/kubernetes-glasses/values.yaml) for the full list.

### Upgrade / uninstall

```bash
helm upgrade kubernetes-glasses ./charts/kubernetes-glasses -n kubernetes-glasses
helm uninstall kubernetes-glasses -n kubernetes-glasses
```

---

## Install with raw manifests (no Helm)

If you don't use Helm, apply the manifests in `deploy/`:

```bash
kubectl apply -f deploy/
```

This creates the `kubernetes-glasses` namespace, RBAC, controller, frontend, and daemonset.

---

## Local development (Docker Compose)

Run the controller + frontend against your current kubeconfig without deploying into the cluster:

```bash
docker compose up --build
```

- Controller API → http://localhost:8080
- Frontend UI → http://localhost:3000

Your `~/.kube/config` is mounted read-only so the controller can reach the cluster. (The DaemonSet only runs in-cluster and is not part of the compose setup.)

---

## Architecture

```
kubernetes-glasses/
├── controller/     # Watches the K8s API, aggregates cluster state, serves it (port 8080)
├── frontend/       # Leptos web UI (port 3000) — cluster, nodes, pods, services, topology
├── daemonset/      # Optional per-node agent → reports node-level data to the controller (port 9090)
├── shared/         # Shared types + wire protocol
├── charts/         # Helm chart
└── deploy/         # Raw Kubernetes manifests (Helm-free install)
```

**How the pieces talk:**

- The **controller** watches the API server (`nodes`, `pods`, `services`, `deployments`, ingresses, etc. — read-only) and holds the aggregated cluster state.
- The **frontend** calls the controller over HTTP and renders the dashboard.
- The **daemonset** runs one pod per node, connects back to the controller over a WebSocket, and streams node-local details.

### RBAC

The chart creates two least-privilege ClusterRoles:

- **controller** — `get/list/watch` on core resources (nodes, pods, services, namespaces, events, endpoints), apps workloads, and networking resources
- **daemonset** — `get/list` on nodes (incl. `nodes/proxy`, `nodes/stats`, `nodes/metrics`) and pods

Everything is read-only. kubernetes-glasses never mutates your cluster.

---

## Prerequisites

- A Kubernetes cluster and `kubectl` configured to reach it
- **Helm 3** (for the Helm install path)
- Permission to create a namespace and cluster-scoped RBAC (ClusterRole/ClusterRoleBinding)

---

## License

Copyright (c) 2026 Stillwater Strategic Solutions LLC. All rights reserved.

This is **source-available** software, not open source. It is free for
personal, non-commercial evaluation and testing. **Commercial and production
use requires a paid license.** See [LICENSE](LICENSE) for full terms, or
contact Stillwater Strategic Solutions LLC at michaelisaacs121092@gmail.com
for commercial licensing.
