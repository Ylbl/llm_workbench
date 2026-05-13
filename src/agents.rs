use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentConfigRow {
    pub id: Uuid,
    pub workspace_item_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub llm_request_profile_id: Option<Uuid>,
    pub system_prompt: Option<String>,
    pub selected_prompt_block_ids: Vec<Uuid>,
    pub tool_permissions: Value,
    pub runtime_config: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentRunRow {
    pub id: Uuid,
    pub agent_config_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub status: String,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub llm_request_profile_id: Option<Uuid>,
    #[serde(default)]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub selected_prompt_block_ids: Vec<Uuid>,
    #[serde(default)]
    pub tool_permissions: Option<Value>,
    #[serde(default)]
    pub runtime_config: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub enabled: Option<bool>,
    pub llm_request_profile_id: Option<Option<Uuid>>,
    pub system_prompt: Option<Option<String>>,
    pub selected_prompt_block_ids: Option<Vec<Uuid>>,
    pub tool_permissions: Option<Value>,
    pub runtime_config: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct RunAgentRequest {
    pub input: Value,
    #[serde(default)]
    pub conversation_id: Option<Uuid>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/agents", get(list_agents).post(create_agent))
        .route("/api/agents/{id}", get(get_agent).patch(update_agent).delete(delete_agent))
        .route("/api/agents/{id}/run", post(run_agent))
        .route("/api/agents/{id}/runs", get(list_agent_runs))
}

fn validate_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() { Err(ApiError::validation("name empty")) } else { Ok(()) }
}

async fn list_agents(State(state): State<AppState>) -> Result<Json<Vec<AgentConfigRow>>, ApiError> {
    let rows = sqlx::query_as::<_, AgentConfigRow>(
        "SELECT id, workspace_item_id, name, description, enabled, llm_request_profile_id,
                system_prompt, selected_prompt_block_ids, tool_permissions, runtime_config, created_at, updated_at
         FROM agent_configs ORDER BY created_at",
    ).fetch_all(state.database()?).await.map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn create_agent(State(state): State<AppState>, Json(payload): Json<CreateAgentRequest>)
    -> Result<(axum::http::StatusCode, Json<AgentConfigRow>), ApiError> {
    validate_name(&payload.name)?;
    let pool = state.database()?;

    let workspace_item_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO workspace_items (item_type, title, sort_order) VALUES ('agent_config', $1, 0) RETURNING id",
    ).bind(payload.name.trim()).fetch_one(pool).await.map_err(ApiError::from)?;

    let tp = payload.tool_permissions.unwrap_or(serde_json::json!({}));
    let rc = payload.runtime_config.unwrap_or(serde_json::json!({}));

    let row = sqlx::query_as::<_, AgentConfigRow>(
        "INSERT INTO agent_configs (workspace_item_id, name, description, llm_request_profile_id, system_prompt, selected_prompt_block_ids, tool_permissions, runtime_config)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         RETURNING id, workspace_item_id, name, description, enabled, llm_request_profile_id, system_prompt, selected_prompt_block_ids, tool_permissions, runtime_config, created_at, updated_at",
    ).bind(workspace_item_id).bind(payload.name.trim()).bind(&payload.description)
     .bind(payload.llm_request_profile_id).bind(&payload.system_prompt)
     .bind(&payload.selected_prompt_block_ids).bind(&tp).bind(&rc)
     .fetch_one(pool).await.map_err(ApiError::from)?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn get_agent(State(state): State<AppState>, Path(id): Path<Uuid>) -> Result<Json<AgentConfigRow>, ApiError> {
    let row = sqlx::query_as::<_, AgentConfigRow>(
        "SELECT * FROM agent_configs WHERE id=$1",
    ).bind(id).fetch_optional(state.database()?).await.map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found("agent not found"))?;
    Ok(Json(row))
}

async fn update_agent(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<UpdateAgentRequest>)
    -> Result<Json<AgentConfigRow>, ApiError> {
    let existing = get_agent(State(state.clone()), Path(id)).await?;
    let name = match payload.name { Some(n) => { validate_name(&n)?; n } None => existing.0.name };
    let desc = payload.description.unwrap_or(existing.0.description);
    let enabled = payload.enabled.unwrap_or(existing.0.enabled);
    let rp = payload.llm_request_profile_id.unwrap_or(existing.0.llm_request_profile_id);
    let sp = payload.system_prompt.unwrap_or(existing.0.system_prompt);
    let pb = payload.selected_prompt_block_ids.unwrap_or(existing.0.selected_prompt_block_ids);
    let tp = payload.tool_permissions.unwrap_or(existing.0.tool_permissions);
    let rc = payload.runtime_config.unwrap_or(existing.0.runtime_config);

    let row = sqlx::query_as::<_, AgentConfigRow>(
        "UPDATE agent_configs SET name=$1,description=$2,enabled=$3,llm_request_profile_id=$4,system_prompt=$5,selected_prompt_block_ids=$6,tool_permissions=$7,runtime_config=$8,updated_at=now()
         WHERE id=$9 RETURNING *",
    ).bind(&name).bind(&desc).bind(enabled).bind(rp).bind(&sp).bind(&pb).bind(&tp).bind(&rc).bind(id)
     .fetch_one(state.database()?).await.map_err(ApiError::from)?;
    if let Some(wid) = existing.0.workspace_item_id {
        let _ = sqlx::query("UPDATE workspace_items SET title=$1, updated_at=now() WHERE id=$2")
            .bind(&name).bind(wid).execute(state.database()?).await;
    }
    Ok(Json(row))
}

async fn delete_agent(State(state): State<AppState>, Path(id): Path<Uuid>)
    -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    let r = sqlx::query("DELETE FROM agent_configs WHERE id=$1").bind(id).execute(state.database()?).await.map_err(ApiError::from)?;
    if r.rows_affected() == 0 { return Err(ApiError::not_found("not found")); }
    Ok((axum::http::StatusCode::OK, Json(serde_json::json!({"deleted":true}))))
}

async fn run_agent(State(state): State<AppState>, Path(id): Path<Uuid>, Json(payload): Json<RunAgentRequest>)
    -> Result<Json<AgentRunRow>, ApiError> {
    let pool = state.database()?.clone();

    let agent = get_agent_internal(&pool, id).await?;
    if !agent.enabled {
        return Err(ApiError::validation("agent is disabled"));
    }

    let started_at = Utc::now();
    let run = sqlx::query_as::<_, AgentRunRow>(
        "INSERT INTO agent_runs (agent_config_id, conversation_id, status, input, started_at)
         VALUES ($1,$2,'running',$3,$4)
         RETURNING *",
    ).bind(id).bind(payload.conversation_id).bind(&payload.input).bind(started_at)
     .fetch_one(&pool).await.map_err(ApiError::from)?;

    let (status, output, error) = if let Some(profile_id) = agent.llm_request_profile_id {
        let (tx, _rx) = tokio::sync::mpsc::channel::<Result<axum::response::sse::Event, std::convert::Infallible>>(1);
        match crate::llm::stream_llm_response(
            &pool, payload.conversation_id.unwrap_or(Uuid::nil()), profile_id,
            &agent.selected_prompt_block_ids, agent.system_prompt.as_deref(), tx,
        ).await {
            Ok((content, _req)) => ("completed".to_string(), Some(serde_json::json!({"content": content})), None),
            Err(e) => ("failed".to_string(), None, Some(e.to_string())),
        }
    } else {
        ("completed".to_string(), Some(serde_json::json!({"message": "No profile configured"})), None)
    };

    let finished_at = Utc::now();
    let final_run = sqlx::query_as::<_, AgentRunRow>(
        "UPDATE agent_runs SET status=$1, output=$2, error=$3, finished_at=$4 WHERE id=$5 RETURNING *",
    ).bind(&status).bind(&output).bind(&error).bind(finished_at).bind(run.id)
     .fetch_one(&pool).await.map_err(ApiError::from)?;

    Ok(Json(final_run))
}

async fn get_agent_internal(pool: &sqlx::PgPool, id: Uuid) -> Result<AgentConfigRow, ApiError> {
    sqlx::query_as::<_, AgentConfigRow>("SELECT * FROM agent_configs WHERE id=$1")
        .bind(id).fetch_optional(pool).await.map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("agent not found"))
}

async fn list_agent_runs(State(state): State<AppState>, Path(id): Path<Uuid>)
    -> Result<Json<Vec<AgentRunRow>>, ApiError> {
    let rows = sqlx::query_as::<_, AgentRunRow>(
        "SELECT * FROM agent_runs WHERE agent_config_id=$1 ORDER BY created_at DESC LIMIT 50",
    ).bind(id).fetch_all(state.database()?).await.map_err(ApiError::from)?;
    Ok(Json(rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;
    use crate::{AppConfig, AppState, build_router};

    fn unique(prefix: &str) -> String {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        format!("{prefix}.{nanos}")
    }
    async fn test_pool() -> Option<PgPool> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations").run(&pool).await.expect("migrate");
        Some(pool)
    }

    #[tokio::test]
    async fn agent_crud() {
        let Some(pool) = test_pool().await else { return; };
        let app = build_router(AppState { config: std::sync::Arc::new(AppConfig::for_test()), db: Some(pool.clone()) });
        let nm = unique("agent");
        let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/agents")
            .header("content-type","application/json")
            .body(Body::from(json!({"name":nm,"system_prompt":"You are helpful"}).to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let a: Value = serde_json::from_slice(&b).unwrap();
        let id = a["id"].as_str().unwrap();

        let resp = app.clone().oneshot(Request::builder().uri(format!("/api/agents/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app.clone().oneshot(Request::builder().method("DELETE").uri(format!("/api/agents/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
