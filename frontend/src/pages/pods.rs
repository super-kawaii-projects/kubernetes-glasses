use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

/// Pod list with namespace filtering, status indicators, resource usage.
#[component]
pub fn PodsPage() -> impl IntoView {
    view! {
        <div class="page pods-page">
            <h1>"Pods"</h1>

            <div class="filter-bar">
                <input type="text" placeholder="Filter by namespace..." class="filter-input"/>
                <input type="text" placeholder="Search pods..." class="filter-input"/>
            </div>

            <table class="resource-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Namespace"</th>
                        <th>"Status"</th>
                        <th>"Node"</th>
                        <th>"CPU"</th>
                        <th>"Memory"</th>
                        <th>"Restarts"</th>
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

/// Single pod detail — containers, logs, events, resource usage.
#[component]
pub fn PodDetailPage() -> impl IntoView {
    let params = use_params_map();
    let pod_ns = move || {
        params.read().get("namespace").unwrap_or_default()
    };
    let pod_name = move || {
        params.read().get("name").unwrap_or_default()
    };

    view! {
        <div class="page pod-detail-page">
            <h1>{move || format!("{}/{}", pod_ns(), pod_name())}</h1>

            <section class="pod-info">
                <h2>"Pod Info"</h2>
                <p class="placeholder">"Loading..."</p>
            </section>

            <section class="pod-containers">
                <h2>"Containers"</h2>
                <p class="placeholder">"Loading containers..."</p>
            </section>

            <section class="pod-logs">
                <h2>"Logs"</h2>
                <pre class="log-output">"Connect to view logs..."</pre>
            </section>

            <section class="pod-events">
                <h2>"Events"</h2>
                <p class="placeholder">"Loading events..."</p>
            </section>
        </div>
    }
}
