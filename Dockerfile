# kubernetes-glasses — Multi-stage Docker Build
# Builds: controller, daemonset, frontend (separate targets)

FROM rust:1.82-bookworm AS builder

RUN rustup target add wasm32-unknown-unknown && \
    cargo install cargo-leptos

WORKDIR /app
COPY Cargo.toml rust-toolchain.toml ./
COPY shared/ shared/
COPY controller/ controller/
COPY daemonset/ daemonset/
COPY frontend/ frontend/

# Build all binaries
RUN cargo build --release -p controller -p daemonset
RUN cargo leptos build --release --project frontend

# ─── Controller image ────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS controller

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash kglass && mkdir -p /app && chown -R kglass:kglass /app
USER kglass
WORKDIR /app

COPY --from=builder --chown=kglass:kglass /app/target/release/controller ./
EXPOSE 8080
HEALTHCHECK --interval=15s --timeout=3s CMD curl -f http://localhost:8080/healthz || exit 1
ENTRYPOINT ["./controller"]

# ─── DaemonSet image ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS daemonset

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash kglass && mkdir -p /app && chown -R kglass:kglass /app
USER kglass
WORKDIR /app

COPY --from=builder --chown=kglass:kglass /app/target/release/daemonset ./
EXPOSE 9090
HEALTHCHECK --interval=10s --timeout=3s CMD curl -f http://localhost:9090/healthz || exit 1
ENTRYPOINT ["./daemonset"]

# ─── Frontend image ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS frontend

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

RUN useradd -m -s /bin/bash kglass && mkdir -p /app && chown -R kglass:kglass /app
USER kglass
WORKDIR /app

COPY --from=builder --chown=kglass:kglass /app/target/release/kubernetes-glasses-frontend ./
COPY --from=builder --chown=kglass:kglass /app/target/site ./target/site
EXPOSE 3000

ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="target/site"

HEALTHCHECK --interval=15s --timeout=3s CMD curl -f http://localhost:3000/ || exit 1
ENTRYPOINT ["./kubernetes-glasses-frontend"]
