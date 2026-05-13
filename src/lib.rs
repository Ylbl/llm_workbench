pub mod agents;
pub mod config;
pub mod conversations;
pub mod db;
pub mod error;
mod health;
pub mod llm;
pub mod notes;
pub mod prompt_blocks;
pub mod settings;
pub mod state;
pub mod workspace;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

pub use config::AppConfig;
pub use error::{ApiError, ErrorBody, ErrorResponse};
pub use state::{AppState, StateInitError};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .merge(settings::routes())
        .merge(workspace::routes())
        .merge(notes::routes())
        .merge(conversations::routes())
        .merge(llm::routes())
        .merge(prompt_blocks::routes())
        .merge(agents::routes())
        .fallback(error::not_found)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("llm_workbench=debug,tower_http=info"));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

pub async fn serve(state: AppState) -> std::io::Result<()> {
    let address = state.config.socket_addr();
    let app = build_router(state);
    let listener = TcpListener::bind(address).await?;
    info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_returns_service_status() {
        let app = build_router(AppState::new_without_db(AppConfig::for_test()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["service"], "llm_workbench");
        assert_eq!(payload["status"], "ok");
        assert_eq!(payload["database"]["status"], "not_configured");
    }

    #[tokio::test]
    async fn missing_route_uses_json_error_shape() {
        let app = build_router(AppState::new_without_db(AppConfig::for_test()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/missing")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["error"]["code"], "not_found");
    }
}
