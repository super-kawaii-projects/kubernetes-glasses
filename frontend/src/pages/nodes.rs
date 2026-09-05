use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// List of all nodes with status, roles, resource usage.
#[component]
pub fn NodesPage() -> impl IntoView {
    view! {
        <div class="page nodes-page">
            <h1>"Nodes"</h1>
            <table class="resource-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Status"</th>
                        <th>"Roles"</th>
                        <th>"CPU"</th>
                        <th>"Memory"</th>
                        <th>"Pods"</th>
                        <th>"Version"</th>
                        <th>"Age"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td colspan="8" class="placeholder">
                            "Waiting for cluster data..."
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}

/// Single node detail — resource utilization, conditions, pods running on it.
#[component]
pub fn NodeDetailPage() -> impl IntoView {
    let params = use_params_map();
    let node_name = move || {
        params.read().get("name").unwrap_or_default()
    };

    view! {
        <div class="page node-detail-page">
            <h1>{move || format!("Node: {}", node_name())}</h1>

            <section class="node-info">
                <h2>"Info"</h2>
                <p class="placeholder">"Loading node details..."</p>
            </section>

            <section class="node-resources">
                <h2>"Resource Usage"</h2>
                <div class="resource-bars">
                    <div class="resource-bar">
                        <label>"CPU"</label>
                        <div class="bar"><div class="fill" style="width: 0%"></div></div>
                    </div>
                    <div class="resource-bar">
                        <label>"Memory"</label>
                        <div class="bar"><div class="fill" style="width: 0%"></div></div>
                    </div>
                </div>
            </section>

            <section class="node-pods">
                <h2>"Pods on this Node"</h2>
                <p class="placeholder">"Loading pods..."</p>
            </section>

            <section class="node-conditions">
                <h2>"Conditions"</h2>
                <p class="placeholder">"Loading conditions..."</p>
            </section>
        </div>
    }
}
