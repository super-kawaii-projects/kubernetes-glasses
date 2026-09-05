use leptos::prelude::*;

/// Top-level cluster overview — node count, pod count, health summary.
#[component]
pub fn ClusterPage() -> impl IntoView {
    view! {
        <div class="page cluster-page">
            <h1>"Cluster Overview"</h1>
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-label">"Nodes"</div>
                    <div class="stat-value">"—"</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Pods"</div>
                    <div class="stat-value">"—"</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Services"</div>
                    <div class="stat-value">"—"</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Deployments"</div>
                    <div class="stat-value">"—"</div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">"Namespaces"</div>
                    <div class="stat-value">"—"</div>
                </div>
                <div class="stat-card health">
                    <div class="stat-label">"Cluster Health"</div>
                    <div class="stat-value">"—"</div>
                </div>
            </div>

            <section class="recent-events">
                <h2>"Recent Events"</h2>
                <p class="placeholder">"Connect to controller to see live events..."</p>
            </section>
        </div>
    }
}
