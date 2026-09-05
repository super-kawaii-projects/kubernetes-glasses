use leptos::prelude::*;

/// Interactive cluster topology visualization.
/// Shows nodes, pods, and services as a connected graph.
#[component]
pub fn TopologyPage() -> impl IntoView {
    view! {
        <div class="page topology-page">
            <h1>"Cluster Topology"</h1>

            <div class="topology-controls">
                <select class="filter-select">
                    <option value="all">"All Namespaces"</option>
                    <option value="default">"default"</option>
                    <option value="kube-system">"kube-system"</option>
                </select>
                <label class="toggle">
                    <input type="checkbox" checked/>
                    " Show Services"
                </label>
                <label class="toggle">
                    <input type="checkbox" checked/>
                    " Show Pods"
                </label>
                <label class="toggle">
                    <input type="checkbox"/>
                    " Show Labels"
                </label>
            </div>

            <div class="topology-canvas" id="topology-graph">
                <p class="placeholder">
                    "Interactive topology graph will render here. "
                    "Connect to the controller WebSocket to populate."
                </p>
            </div>

            <div class="topology-legend">
                <span class="legend-item node">"● Node"</span>
                <span class="legend-item pod">"● Pod"</span>
                <span class="legend-item service">"◆ Service"</span>
                <span class="legend-item deployment">"■ Deployment"</span>
            </div>
        </div>
    }
}
