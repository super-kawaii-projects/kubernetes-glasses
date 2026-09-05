use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    StaticSegment, ParamSegment,
};

use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kubernetes-glasses-frontend.css"/>
        <Title text="kubernetes-glasses"/>

        <Router>
            <nav class="top-nav">
                <div class="nav-brand">
                    <span class="brand-icon">"👓"</span>
                    <span class="brand-text">"kubernetes-glasses"</span>
                </div>
                <div class="nav-links">
                    <A href="/">"Cluster"</A>
                    <A href="/nodes">"Nodes"</A>
                    <A href="/pods">"Pods"</A>
                    <A href="/services">"Services"</A>
                    <A href="/topology">"Topology"</A>
                </div>
            </nav>

            <main class="content">
                <Routes fallback=|| view! { <p>"Page not found."</p> }>
                    <Route path=StaticSegment("") view=ClusterPage/>
                    <Route path=StaticSegment("nodes") view=NodesPage/>
                    <Route path=(StaticSegment("nodes"), ParamSegment("name")) view=NodeDetailPage/>
                    <Route path=StaticSegment("pods") view=PodsPage/>
                    <Route path=(StaticSegment("pods"), ParamSegment("namespace"), ParamSegment("name")) view=PodDetailPage/>
                    <Route path=StaticSegment("services") view=ServicesPage/>
                    <Route path=StaticSegment("topology") view=TopologyPage/>
                </Routes>
            </main>
        </Router>
    }
}
