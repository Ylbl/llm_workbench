use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{ApiError, AppState};

const VALID_ITEM_TYPES: &[&str] = &[
    "note",
    "chat",
    "agent_config",
    "file",
    "task",
    "settings_view",
];

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkspaceItemRow {
    pub id: Uuid,
    pub item_type: String,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceItemRequest {
    pub item_type: String,
    pub title: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default = "default_sort_order")]
    pub sort_order: i32,
    #[serde(default = "default_metadata")]
    pub metadata: Value,
}

fn default_sort_order() -> i32 {
    0
}

fn default_metadata() -> Value {
    Value::Object(Default::default())
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceItemRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub parent_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Clone)]
pub struct WorkspaceService {
    pool: PgPool,
}

impl WorkspaceService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<WorkspaceItemRow>, ApiError> {
        list_items_from_pool(&self.pool).await
    }

    pub async fn create(
        &self,
        req: CreateWorkspaceItemRequest,
    ) -> Result<WorkspaceItemRow, ApiError> {
        create_item_in_pool(&self.pool, req).await
    }

    pub async fn get(&self, id: Uuid) -> Result<WorkspaceItemRow, ApiError> {
        get_item_from_pool(&self.pool, id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateWorkspaceItemRequest,
    ) -> Result<WorkspaceItemRow, ApiError> {
        update_item_in_pool(&self.pool, id, req).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        delete_item_from_pool(&self.pool, id).await
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/workspace/items", get(list_items).post(create_item))
        .route(
            "/api/workspace/items/{id}",
            get(get_item).patch(update_item).delete(delete_item),
        )
}

async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceItemRow>>, ApiError> {
    let items = list_items_from_pool(state.database()?).await?;
    Ok(Json(items))
}

async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkspaceItemRequest>,
) -> Result<(axum::http::StatusCode, Json<WorkspaceItemRow>), ApiError> {
    let item = create_item_in_pool(state.database()?, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(item)))
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkspaceItemRow>, ApiError> {
    let item = get_item_from_pool(state.database()?, id).await?;
    Ok(Json(item))
}

async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateWorkspaceItemRequest>,
) -> Result<Json<WorkspaceItemRow>, ApiError> {
    let item = update_item_in_pool(state.database()?, id, payload).await?;
    Ok(Json(item))
}

async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    delete_item_from_pool(state.database()?, id).await?;
    Ok((
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "deleted": true })),
    ))
}

fn validate_item_type(item_type: &str) -> Result<(), ApiError> {
    if item_type.trim().is_empty() {
        return Err(ApiError::validation("item_type must not be empty"));
    }

    if !VALID_ITEM_TYPES.contains(&item_type) {
        return Err(ApiError::validation(format!(
            "Invalid item_type '{}'. Must be one of: {}",
            item_type,
            VALID_ITEM_TYPES.join(", ")
        )));
    }

    Ok(())
}

fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::validation("title must not be empty"));
    }

    Ok(())
}

async fn list_items_from_pool(pool: &PgPool) -> Result<Vec<WorkspaceItemRow>, ApiError> {
    let items = sqlx::query_as::<_, WorkspaceItemRow>(
        "SELECT id, item_type, title, parent_id, sort_order, metadata, created_at, updated_at
         FROM workspace_items
         ORDER BY sort_order, created_at",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(items)
}

async fn create_item_in_pool(
    pool: &PgPool,
    req: CreateWorkspaceItemRequest,
) -> Result<WorkspaceItemRow, ApiError> {
    validate_item_type(&req.item_type)?;
    validate_title(&req.title)?;

    let item = sqlx::query_as::<_, WorkspaceItemRow>(
        "INSERT INTO workspace_items (item_type, title, parent_id, sort_order, metadata)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, item_type, title, parent_id, sort_order, metadata, created_at, updated_at",
    )
    .bind(&req.item_type)
    .bind(req.title.trim())
    .bind(req.parent_id)
    .bind(req.sort_order)
    .bind(&req.metadata)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(item)
}

async fn get_item_from_pool(pool: &PgPool, id: Uuid) -> Result<WorkspaceItemRow, ApiError> {
    let item = sqlx::query_as::<_, WorkspaceItemRow>(
        "SELECT id, item_type, title, parent_id, sort_order, metadata, created_at, updated_at
         FROM workspace_items
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Workspace item {id} not found")))?;

    Ok(item)
}

async fn update_item_in_pool(
    pool: &PgPool,
    id: Uuid,
    req: UpdateWorkspaceItemRequest,
) -> Result<WorkspaceItemRow, ApiError> {
    let existing = get_item_from_pool(pool, id).await?;

    let title = match req.title {
        Some(ref t) => {
            validate_title(t)?;
            t.trim().to_string()
        }
        None => existing.title,
    };

    let parent_id = match req.parent_id {
        Some(parent) => parent,
        None => existing.parent_id,
    };

    let sort_order = req.sort_order.unwrap_or(existing.sort_order);

    let metadata = req.metadata.unwrap_or(existing.metadata);

    let item = sqlx::query_as::<_, WorkspaceItemRow>(
        "UPDATE workspace_items
         SET title = $1, parent_id = $2, sort_order = $3, metadata = $4, updated_at = now()
         WHERE id = $5
         RETURNING id, item_type, title, parent_id, sort_order, metadata, created_at, updated_at",
    )
    .bind(&title)
    .bind(parent_id)
    .bind(sort_order)
    .bind(&metadata)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Workspace item {id} not found")))?;

    Ok(item)
}

async fn delete_item_from_pool(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
    let result = sqlx::query("DELETE FROM workspace_items WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(format!(
            "Workspace item {id} not found"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    use crate::{AppConfig, AppState, build_router};

    fn unique_title(prefix: &str) -> String {
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

    async fn delete_item(pool: &PgPool, id: Uuid) {
        sqlx::query("DELETE FROM workspace_items WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .expect("delete test workspace item");
    }

    #[tokio::test]
    async fn workspace_endpoint_reports_database_error_without_pool() {
        let app = build_router(AppState::new_without_db(AppConfig::for_test()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/workspace/items")
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
    async fn workspace_service_crud_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let service = WorkspaceService::new(pool.clone());
        let title = unique_title("ws.service");

        let created = service
            .create(CreateWorkspaceItemRequest {
                item_type: "note".into(),
                title: title.clone(),
                parent_id: None,
                sort_order: 0,
                metadata: json!({}),
            })
            .await
            .unwrap();

        assert_eq!(created.item_type, "note");
        assert_eq!(created.title, title);

        let fetched = service.get(created.id).await.unwrap();
        assert_eq!(fetched.id, created.id);

        let items = service.list().await.unwrap();
        assert!(items.iter().any(|i| i.id == created.id));

        let updated = service
            .update(
                created.id,
                UpdateWorkspaceItemRequest {
                    title: Some(format!("{title}.updated")),
                    parent_id: None,
                    sort_order: None,
                    metadata: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.title, format!("{title}.updated"));

        service.delete(created.id).await.unwrap();

        let result = service.get(created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn workspace_parent_child_ordering_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let service = WorkspaceService::new(pool.clone());
        let parent_title = unique_title("ws.parent");

        let parent = service
            .create(CreateWorkspaceItemRequest {
                item_type: "note".into(),
                title: parent_title.clone(),
                parent_id: None,
                sort_order: 0,
                metadata: json!({}),
            })
            .await
            .unwrap();

        let child_title = unique_title("ws.child");

        let child = service
            .create(CreateWorkspaceItemRequest {
                item_type: "note".into(),
                title: child_title.clone(),
                parent_id: Some(parent.id),
                sort_order: 1,
                metadata: json!({}),
            })
            .await
            .unwrap();

        assert_eq!(child.parent_id, Some(parent.id));

        let items = service.list().await.unwrap();
        let child_from_list = items.iter().find(|i| i.id == child.id).unwrap();
        assert_eq!(child_from_list.parent_id, Some(parent.id));

        service.delete(child.id).await.unwrap();
        service.delete(parent.id).await.unwrap();
    }

    #[tokio::test]
    async fn workspace_item_create_api_returns_201_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let title = unique_title("ws.api.create");

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let request_body = json!({
            "item_type": "note",
            "title": title
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/workspace/items")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["item_type"], "note");
        assert_eq!(payload["title"], title);

        let id: Uuid = serde_json::from_value(payload["id"].clone()).unwrap();
        delete_item(&pool, id).await;
    }

    #[tokio::test]
    async fn workspace_item_delete_api_returns_ok_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let service = WorkspaceService::new(pool.clone());
        let title = unique_title("ws.api.delete");

        let created = service
            .create(CreateWorkspaceItemRequest {
                item_type: "note".into(),
                title: title.clone(),
                parent_id: None,
                sort_order: 0,
                metadata: json!({}),
            })
            .await
            .unwrap();

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/workspace/items/{}", created.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let result = service.get(created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn workspace_item_update_api_patches_fields_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let service = WorkspaceService::new(pool.clone());
        let title = unique_title("ws.api.update");

        let created = service
            .create(CreateWorkspaceItemRequest {
                item_type: "note".into(),
                title: title.clone(),
                parent_id: None,
                sort_order: 0,
                metadata: json!({}),
            })
            .await
            .unwrap();

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let updated_title = format!("{title}.patched");
        let request_body = json!({
            "title": updated_title,
            "sort_order": 42
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/workspace/items/{}", created.id))
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

        assert_eq!(payload["title"], updated_title);
        assert_eq!(payload["sort_order"], 42);

        delete_item(&pool, created.id).await;
    }

    #[tokio::test]
    async fn workspace_item_create_rejects_invalid_item_type_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL workspace test; TEST_DATABASE_URL is not set");
            return;
        };

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let request_body = json!({
            "item_type": "invalid_type",
            "title": "test"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/workspace/items")
                    .header("content-type", "application/json")
                    .body(Body::from(request_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["error"]["code"], "validation_error");
    }
}
