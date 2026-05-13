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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PromptBlockRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub block_type: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBlockRequest {
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_block_type")]
    pub block_type: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_block_type() -> String { "system".into() }

#[derive(Debug, Deserialize)]
pub struct UpdateBlockRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub description: Option<Option<String>>,
    #[serde(default)]
    pub block_type: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/prompt-blocks", get(list_blocks).post(create_block))
        .route("/api/prompt-blocks/{id}", get(get_block).patch(update_block).delete(delete_block))
}

fn validate_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() { Err(ApiError::validation("name must not be empty")) } else { Ok(()) }
}

async fn list_blocks(State(state): State<AppState>) -> Result<Json<Vec<PromptBlockRow>>, ApiError> {
    let rows = sqlx::query_as::<_, PromptBlockRow>(
        "SELECT id, name, description, content, block_type, enabled, sort_order, created_at, updated_at
         FROM prompt_blocks ORDER BY sort_order, created_at",
    ).fetch_all(state.database()?).await.map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn create_block(State(state): State<AppState>, Json(payload): Json<CreateBlockRequest>)
    -> Result<(axum::http::StatusCode, Json<PromptBlockRow>), ApiError> {
    validate_name(&payload.name)?;
    let row = sqlx::query_as::<_, PromptBlockRow>(
        "INSERT INTO prompt_blocks (name, description, content, block_type, enabled, sort_order)
         VALUES ($1,$2,$3,$4,$5,$6)
         RETURNING id, name, description, content, block_type, enabled, sort_order, created_at, updated_at",
    ).bind(payload.name.trim()).bind(&payload.description).bind(&payload.content)
     .bind(&payload.block_type).bind(payload.enabled).bind(payload.sort_order)
     .fetch_one(state.database()?).await.map_err(ApiError::from)?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn get_block(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<PromptBlockRow>, ApiError> {
    let row = sqlx::query_as::<_, PromptBlockRow>(
        "SELECT * FROM prompt_blocks WHERE id=$1"
    ).bind(id).fetch_optional(state.database()?).await.map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found("block not found"))?;
    Ok(Json(row))
}

async fn update_block(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<UpdateBlockRequest>)
    -> Result<Json<PromptBlockRow>, ApiError> {
    let existing = get_block(State(state.clone()), Path(id)).await?;
    let name = match payload.name { Some(n) => { validate_name(&n)?; n } None => existing.0.name };
    let content = payload.content.unwrap_or(existing.0.content);
    let description = payload.description.unwrap_or(existing.0.description);
    let bt = payload.block_type.unwrap_or(existing.0.block_type);
    let enabled = payload.enabled.unwrap_or(existing.0.enabled);
    let so = payload.sort_order.unwrap_or(existing.0.sort_order);

    let row = sqlx::query_as::<_, PromptBlockRow>(
        "UPDATE prompt_blocks SET name=$1,description=$2,content=$3,block_type=$4,enabled=$5,sort_order=$6,updated_at=now()
         WHERE id=$7 RETURNING *",
    ).bind(&name).bind(&description).bind(&content).bind(&bt).bind(enabled).bind(so).bind(id)
     .fetch_one(state.database()?).await.map_err(ApiError::from)?;
    Ok(Json(row))
}

async fn delete_block(State(state): State<AppState>, Path(id): Path<Uuid>)
    -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    let r = sqlx::query("DELETE FROM prompt_blocks WHERE id=$1").bind(id).execute(state.database()?).await.map_err(ApiError::from)?;
    if r.rows_affected() == 0 { return Err(ApiError::not_found("not found")); }
    Ok((axum::http::StatusCode::OK, Json(serde_json::json!({"deleted":true}))))
}

pub async fn inject_prompt_blocks(
    pool: &PgPool,
    block_ids: &[Uuid],
) -> Result<Vec<PromptBlockRow>, ApiError> {
    if block_ids.is_empty() { return Ok(vec![]); }
    let rows = sqlx::query_as::<_, PromptBlockRow>(
        "SELECT * FROM prompt_blocks WHERE id = ANY($1) AND enabled = true ORDER BY sort_order, created_at",
    ).bind(block_ids).fetch_all(pool).await.map_err(ApiError::from)?;
    Ok(rows)
}
