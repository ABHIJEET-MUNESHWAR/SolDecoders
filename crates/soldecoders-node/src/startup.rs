//! Composition root: build the registry, schema, HTTP app, and run the server.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;
use tower_http::trace::TraceLayer;

use soldecoders_api::{build_schema, ApiContext, SolDecodersSchema};
use soldecoders_core::DecoderRegistry;

use crate::config::ServeArgs;
use crate::telemetry;

/// Build the shared, built-in decoder registry.
pub fn build_registry() -> Arc<DecoderRegistry> {
    Arc::new(DecoderRegistry::builtin())
}

/// Build the GraphQL schema around a shared registry.
pub fn build_schema_from_registry(registry: Arc<DecoderRegistry>) -> SolDecodersSchema {
    build_schema(ApiContext::new(registry))
}

/// Assemble the axum application: GraphQL endpoint, health probes, and metrics.
pub fn build_app(schema: SolDecodersSchema, metrics: PrometheusHandle) -> Router {
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/health/live", get(health_live))
        .route("/health/ready", get(health_ready))
        .route("/metrics", get(metrics_handler))
        .layer(Extension(schema))
        .layer(Extension(metrics))
        .layer(TraceLayer::new_for_http())
}

/// Build registry + schema + app and serve until a shutdown signal arrives.
pub async fn run_server(args: ServeArgs) -> Result<()> {
    let metrics = telemetry::init_metrics()?;
    let registry = build_registry();
    let schema = build_schema_from_registry(registry);
    let app = build_app(schema, metrics);

    let addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .with_context(|| format!("invalid bind address {}:{}", args.host, args.port))?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!(%addr, "soldecoders GraphQL API listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;
    tracing::info!("shutdown complete");
    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

async fn graphql_handler(
    Extension(schema): Extension<SolDecodersSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    metrics::counter!("soldecoders_graphql_requests_total").increment(1);
    schema.execute(req.into_inner()).await.into()
}

async fn health_live() -> impl IntoResponse {
    Json(json!({ "status": "live" }))
}

async fn health_ready() -> impl IntoResponse {
    Json(json!({ "status": "ready" }))
}

async fn metrics_handler(Extension(handle): Extension<PrometheusHandle>) -> impl IntoResponse {
    handle.render()
}

/// Resolve when the process receives Ctrl-C or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            sig.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received Ctrl-C, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use metrics_exporter_prometheus::PrometheusBuilder;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let metrics = PrometheusBuilder::new().build_recorder().handle();
        let schema = build_schema_from_registry(build_registry());
        build_app(schema, metrics)
    }

    #[tokio::test]
    async fn health_ready_returns_ok() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_endpoint_renders() {
        let app = test_app();
        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn graphql_supported_programs_over_http() {
        let app = test_app();
        let body =
            serde_json::to_vec(&json!({ "query": "{ supportedPrograms { name } }" })).unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
