use std::collections::BTreeMap;

use axum::{
    Json, Router,
    extract::State,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::{ApiError, AppState};

#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub settings: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct SettingsPatchRequest {
    pub settings: BTreeMap<String, Value>,
}

#[derive(Clone)]
pub struct SettingsService {
    pool: PgPool,
}

impl SettingsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<BTreeMap<String, Value>, ApiError> {
        list_settings_from_pool(&self.pool).await
    }

    pub async fn get_one(&self, key: &str) -> Result<Option<Value>, ApiError> {
        get_setting_from_pool(&self.pool, key).await
    }

    pub async fn upsert(&self, key: &str, value: Value) -> Result<(), ApiError> {
        upsert_setting_into_pool(&self.pool, key, value).await
    }

    pub async fn patch_many(
        &self,
        settings: BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, Value>, ApiError> {
        patch_settings_into_pool(&self.pool, settings).await
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/settings", get(list_settings).patch(patch_settings))
}

async fn list_settings(State(state): State<AppState>) -> Result<Json<SettingsResponse>, ApiError> {
    let settings = list_settings_from_pool(state.database()?).await?;
    Ok(Json(SettingsResponse { settings }))
}

async fn patch_settings(
    State(state): State<AppState>,
    Json(payload): Json<SettingsPatchRequest>,
) -> Result<Json<SettingsResponse>, ApiError> {
    let settings = patch_settings_into_pool(state.database()?, payload.settings).await?;
    Ok(Json(SettingsResponse { settings }))
}

async fn list_settings_from_pool(pool: &PgPool) -> Result<BTreeMap<String, Value>, ApiError> {
    let rows = sqlx::query("SELECT key, value FROM settings ORDER BY key")
        .fetch_all(pool)
        .await
        .map_err(ApiError::from)?;

    let mut settings = BTreeMap::new();
    for row in rows {
        let key: String = row.try_get("key").map_err(ApiError::from)?;
        let value: Value = row.try_get("value").map_err(ApiError::from)?;
        settings.insert(key, value);
    }

    Ok(settings)
}

async fn get_setting_from_pool(pool: &PgPool, key: &str) -> Result<Option<Value>, ApiError> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(ApiError::from)?;

    row.map(|row| row.try_get("value").map_err(ApiError::from))
        .transpose()
}

async fn upsert_setting_into_pool(pool: &PgPool, key: &str, value: Value) -> Result<(), ApiError> {
    validate_key(key)?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES ($1, $2, now())
        ON CONFLICT (key)
        DO UPDATE SET value = EXCLUDED.value, updated_at = now()
        "#,
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(())
}

async fn patch_settings_into_pool(
    pool: &PgPool,
    settings: BTreeMap<String, Value>,
) -> Result<BTreeMap<String, Value>, ApiError> {
    for (key, value) in settings {
        upsert_setting_into_pool(pool, &key, value).await?;
    }

    list_settings_from_pool(pool).await
}

fn validate_key(key: &str) -> Result<(), ApiError> {
    if key.trim().is_empty() {
        return Err(ApiError::validation("Setting key must not be empty"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{Value, json};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    use crate::{AppConfig, AppState, build_router};

    fn unique_key(prefix: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{prefix}.{nanos}")
    }

    async fn test_pool() -> Option<PgPool> {
        let database_url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("connect TEST_DATABASE_URL");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        Some(pool)
    }

    async fn delete_setting(pool: &PgPool, key: &str) {
        sqlx::query("DELETE FROM settings WHERE key = $1")
            .bind(key)
            .execute(pool)
            .await
            .expect("delete test setting");
    }

    #[tokio::test]
    async fn settings_endpoint_reports_database_error_without_pool() {
        let app = build_router(AppState::new_without_db(AppConfig::for_test()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/settings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["error"]["code"], "database_error");
    }

    #[tokio::test]
    async fn settings_service_upserts_and_reads_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL settings service test; TEST_DATABASE_URL is not set");
            return;
        };

        let key = unique_key("settings.service");
        let service = SettingsService::new(pool.clone());

        service
            .upsert(&key, json!({ "mode": "dark", "compact": true }))
            .await
            .unwrap();

        let value = service.get_one(&key).await.unwrap();
        assert_eq!(value, Some(json!({ "mode": "dark", "compact": true })));

        delete_setting(&pool, &key).await;
    }

    #[tokio::test]
    async fn settings_patch_api_upserts_multiple_values_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL settings API test; TEST_DATABASE_URL is not set");
            return;
        };

        let first_key = unique_key("settings.api.first");
        let second_key = unique_key("settings.api.second");

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let request_body = json!({
            "settings": {
                first_key.clone(): { "enabled": true },
                second_key.clone(): ["alpha", "beta"]
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["settings"][&first_key], json!({ "enabled": true }));
        assert_eq!(payload["settings"][&second_key], json!(["alpha", "beta"]));

        delete_setting(&pool, &first_key).await;
        delete_setting(&pool, &second_key).await;
    }
}
