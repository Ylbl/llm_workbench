use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub service: &'static str,
    pub version: &'static str,
    pub status: &'static str,
    pub app: AppHealth,
    pub database: DatabaseHealth,
}

#[derive(Debug, Serialize)]
pub struct AppHealth {
    pub host: String,
    pub port: u16,
    pub app_data_dir: String,
}

#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    pub configured: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let config = state.config.clone();
    let database = database_health(&state).await;

    Json(HealthResponse {
        service: "llm_workbench",
        version: env!("CARGO_PKG_VERSION"),
        status: if database.status == "error" {
            "degraded"
        } else {
            "ok"
        },
        app: AppHealth {
            host: config.host.to_string(),
            port: config.port,
            app_data_dir: config.app_data_dir.display().to_string(),
        },
        database,
    })
}

async fn database_health(state: &AppState) -> DatabaseHealth {
    let Some(pool) = state.db.as_ref() else {
        return DatabaseHealth {
            configured: false,
            status: "not_configured".to_string(),
            error: None,
        };
    };

    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(_) => DatabaseHealth {
            configured: true,
            status: "ok".to_string(),
            error: None,
        },
        Err(error) => DatabaseHealth {
            configured: true,
            status: "error".to_string(),
            error: Some(error.to_string()),
        },
    }
}
