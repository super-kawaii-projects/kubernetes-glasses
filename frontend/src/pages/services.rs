use leptos::prelude::*;

/// Service list with type, ports, selectors, endpoints.
#[component]
pub fn ServicesPage() -> impl IntoView {
    view! {
        <div class="page services-page">
            <h1>"Services"</h1>

            <div class="filter-bar">
                <input type="text" placeholder="Filter by namespace..." class="filter-input"/>
                <select class="filter-select">
                    <option value="">"All Types"</option>
                    <option value="ClusterIP">"ClusterIP"</option>
                    <option value="NodePort">"NodePort"</option>
                    <option value="LoadBalancer">"LoadBalancer"</option>
                </select>
            </div>

            <table class="resource-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Namespace"</th>
                        <th>"Type"</th>
                        <th>"Cluster IP"</th>
                        <th>"Ports"</th>
                        <th>"Endpoints"</th>
                        <th>"Age"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td colspan="7" class="placeholder">
                            "Waiting for cluster data..."
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
