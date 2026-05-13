use axum::{
    Json, Router,
    extract::{Path, State},
    response::sse::Event,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::convert::Infallible;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{ApiError, AppState};
use crate::prompt_blocks::PromptBlockRow;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LlmProviderRow {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub default_model: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LlmRequestProfileRow {
    pub id: Uuid,
    pub name: String,
    pub provider_id: Option<Uuid>,
    pub endpoint_path: String,
    pub method: String,
    pub base_body: Value,
    pub headers: Value,
    pub message_injection_mode: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MessageLlmCallRow {
    pub id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub agent_run_id: Option<Uuid>,
    pub provider_id: Option<Uuid>,
    pub request_profile_id: Option<Uuid>,
    pub request_body: Value,
    pub response_body: Option<Value>,
    pub status: String,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub api_key: Option<Option<String>>,
    #[serde(default)]
    pub default_model: Option<Option<String>>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub name: String,
    #[serde(default)]
    pub provider_id: Option<Uuid>,
    #[serde(default = "default_endpoint")]
    pub endpoint_path: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_empty_json")]
    pub base_body: Value,
    #[serde(default = "default_empty_json")]
    pub headers: Value,
    #[serde(default = "default_injection_mode")]
    pub message_injection_mode: String,
}

fn default_endpoint() -> String {
    "/chat/completions".into()
}
fn default_method() -> String {
    "POST".into()
}
fn default_empty_json() -> Value {
    serde_json::json!({})
}
fn default_injection_mode() -> String {
    "replace_messages".into()
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub provider_id: Option<Option<Uuid>>,
    #[serde(default)]
    pub endpoint_path: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub base_body: Option<Value>,
    #[serde(default)]
    pub headers: Option<Value>,
    #[serde(default)]
    pub message_injection_mode: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
}

fn validate_name(name: &str) -> Result<(), ApiError> {
    if name.trim().is_empty() {
        return Err(ApiError::validation("name must not be empty"));
    }
    Ok(())
}

pub fn build_llm_request(
    profile: &LlmRequestProfileRow,
    messages: &[Value],
    prompt_blocks: &[PromptBlockRow],
    system_prompt: Option<&str>,
    runtime_overrides: Option<&Value>,
    raw_body_overrides: Option<&Value>,
) -> Value {
    let mut body = profile.base_body.clone();
    if profile.message_injection_mode == "replace_messages" {
        let mut msg_vec: Vec<Value> = Vec::new();
        if let Some(sp) = system_prompt {
            if !sp.trim().is_empty() {
                msg_vec.push(serde_json::json!({
                    "role": "system",
                    "content": sp,
                }));
            }
        }
        for block in prompt_blocks {
            msg_vec.push(serde_json::json!({
                "role": block.block_type,
                "content": block.content,
            }));
        }
        msg_vec.extend_from_slice(messages);
        body["messages"] = serde_json::json!(msg_vec);
    }
    if let Some(overrides) = runtime_overrides {
        merge_json(&mut body, overrides);
    }
    if let Some(raw) = raw_body_overrides {
        if raw.is_object() {
            body = raw.clone();
        }
    }
    body["stream"] = serde_json::json!(true);
    body
}

fn merge_json(target: &mut Value, source: &Value) {
    if let (Some(t_obj), Some(s_obj)) = (target.as_object_mut(), source.as_object()) {
        for (k, v) in s_obj {
            if let Some(existing) = t_obj.get_mut(k) {
                merge_json(existing, v);
            } else {
                t_obj.insert(k.clone(), v.clone());
            }
        }
    } else {
        *target = source.clone();
    }
}

pub async fn stream_llm_response(
    pool: &PgPool,
    conversation_id: Uuid,
    profile_id: Uuid,
    prompt_block_ids: &[Uuid],
    system_prompt: Option<&str>,
    tx: mpsc::Sender<Result<Event, Infallible>>,
) -> Result<(String, Value), ApiError> {
    let profile = sqlx::query_as::<_, LlmRequestProfileRow>(
        "SELECT * FROM llm_request_profiles WHERE id = $1",
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found("profile not found"))?;

    let provider = match profile.provider_id {
        Some(pid) => sqlx::query_as::<_, LlmProviderRow>(
            "SELECT * FROM llm_providers WHERE id = $1",
        )
        .bind(pid)
        .fetch_optional(pool)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::not_found("provider not found"))?,
        None => return Err(ApiError::validation("No provider configured for this profile")),
    };

    let msg_rows = sqlx::query_as::<_, ConversationMessageRow>(
        "SELECT role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    let msgs: Vec<Value> = msg_rows.iter().map(|m| {
        serde_json::json!({ "role": m.role, "content": m.content })
    }).collect();

    let blocks = crate::prompt_blocks::inject_prompt_blocks(pool, prompt_block_ids).await?;

    let body = build_llm_request(&profile, &msgs, &blocks, system_prompt, None, None);

    let url = format!(
        "{}{}",
        provider.base_url.trim_end_matches('/'),
        profile.endpoint_path
    );

    let client = reqwest::Client::new();
    let mut req = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&body);

    if let Some(key) = &provider.api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let resp = req.send().await
        .map_err(|e| ApiError::internal(format!("Provider request failed: {e}")))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(ApiError::internal(format!("Provider {status}: {text}")));
    }

    let mut full_content = String::new();
    let mut buffer = String::new();
    let mut byte_stream = resp.bytes_stream();

    use futures::StreamExt;
    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk.map_err(|e| ApiError::internal(format!("stream error: {e}")))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() { continue; }
            if !line.starts_with("data: ") { continue; }

            let data = &line[6..];
            if data == "[DONE]" {
                return Ok((full_content, body));
            }

            if let Ok(chunk) = serde_json::from_str::<Value>(data) {
                if let Some(delta) = chunk["choices"][0]["delta"]["content"].as_str() {
                    full_content.push_str(delta);
                    let payload = serde_json::json!({
                        "conversation_id": conversation_id,
                        "delta": delta,
                    });
                    let _ = tx.send(Ok(Event::default()
                        .event("llm.delta")
                        .json_data(payload).unwrap())).await;
                }
            }
        }
    }

    Ok((full_content, body))
}

#[derive(Debug, sqlx::FromRow)]
struct ConversationMessageRow {
    role: String,
    content: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/llm/providers",
            get(list_providers).post(create_provider),
        )
        .route(
            "/api/llm/providers/{id}",
            get(get_provider).patch(update_provider).delete(delete_provider),
        )
        .route(
            "/api/llm/request-profiles",
            get(list_profiles).post(create_profile),
        )
        .route(
            "/api/llm/request-profiles/{id}",
            get(get_profile).patch(update_profile).delete(delete_profile),
        )
        .route(
            "/api/llm/request-profiles/{id}/test",
            post(test_profile),
        )
}

async fn list_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<LlmProviderRow>>, ApiError> {
    let rows = sqlx::query_as::<_, LlmProviderRow>(
        "SELECT id, name, base_url, api_key, default_model, enabled, created_at, updated_at
         FROM llm_providers ORDER BY created_at",
    )
    .fetch_all(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn create_provider(
    State(state): State<AppState>,
    Json(payload): Json<CreateProviderRequest>,
) -> Result<(axum::http::StatusCode, Json<LlmProviderRow>), ApiError> {
    validate_name(&payload.name)?;
    let row = sqlx::query_as::<_, LlmProviderRow>(
        "INSERT INTO llm_providers (name, base_url, api_key, default_model)
         VALUES ($1, $2, $3, $4)
         RETURNING id, name, base_url, api_key, default_model, enabled, created_at, updated_at",
    )
    .bind(payload.name.trim())
    .bind(&payload.base_url)
    .bind(&payload.api_key)
    .bind(&payload.default_model)
    .fetch_one(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn get_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LlmProviderRow>, ApiError> {
    let row = sqlx::query_as::<_, LlmProviderRow>(
        "SELECT id, name, base_url, api_key, default_model, enabled, created_at, updated_at
         FROM llm_providers WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(state.database()?)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Provider {id} not found")))?;
    Ok(Json(row))
}

async fn update_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProviderRequest>,
) -> Result<Json<LlmProviderRow>, ApiError> {
    let existing = get_provider(State(state.clone()), Path(id)).await?;
    let name = match payload.name {
        Some(n) => { validate_name(&n)?; n.trim().into() }
        None => existing.0.name,
    };
    let base_url = payload.base_url.unwrap_or(existing.0.base_url);
    let api_key = match payload.api_key {
        Some(k) => k,
        None => existing.0.api_key,
    };
    let default_model = match payload.default_model {
        Some(m) => m,
        None => existing.0.default_model,
    };
    let enabled = payload.enabled.unwrap_or(existing.0.enabled);

    let row = sqlx::query_as::<_, LlmProviderRow>(
        "UPDATE llm_providers SET name=$1, base_url=$2, api_key=$3, default_model=$4, enabled=$5, updated_at=now()
         WHERE id=$6 RETURNING id, name, base_url, api_key, default_model, enabled, created_at, updated_at",
    )
    .bind(&name).bind(&base_url).bind(&api_key).bind(&default_model).bind(enabled).bind(id)
    .fetch_one(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(row))
}

async fn delete_provider(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    let r = sqlx::query("DELETE FROM llm_providers WHERE id=$1").bind(id).execute(state.database()?).await.map_err(ApiError::from)?;
    if r.rows_affected() == 0 {
        return Err(ApiError::not_found("not found"));
    }
    Ok((axum::http::StatusCode::OK, Json(serde_json::json!({"deleted":true}))))
}

async fn list_profiles(
    State(state): State<AppState>,
) -> Result<Json<Vec<LlmRequestProfileRow>>, ApiError> {
    let rows = sqlx::query_as::<_, LlmRequestProfileRow>(
        "SELECT id, name, provider_id, endpoint_path, method, base_body, headers, message_injection_mode, enabled, created_at, updated_at
         FROM llm_request_profiles ORDER BY created_at",
    )
    .fetch_all(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(rows))
}

async fn create_profile(
    State(state): State<AppState>,
    Json(payload): Json<CreateProfileRequest>,
) -> Result<(axum::http::StatusCode, Json<LlmRequestProfileRow>), ApiError> {
    validate_name(&payload.name)?;
    let row = sqlx::query_as::<_, LlmRequestProfileRow>(
        "INSERT INTO llm_request_profiles (name, provider_id, endpoint_path, method, base_body, headers, message_injection_mode)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, name, provider_id, endpoint_path, method, base_body, headers, message_injection_mode, enabled, created_at, updated_at",
    )
    .bind(payload.name.trim())
    .bind(payload.provider_id)
    .bind(&payload.endpoint_path)
    .bind(&payload.method)
    .bind(&payload.base_body)
    .bind(&payload.headers)
    .bind(&payload.message_injection_mode)
    .fetch_one(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok((axum::http::StatusCode::CREATED, Json(row)))
}

async fn get_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<LlmRequestProfileRow>, ApiError> {
    let row = sqlx::query_as::<_, LlmRequestProfileRow>(
        "SELECT id, name, provider_id, endpoint_path, method, base_body, headers, message_injection_mode, enabled, created_at, updated_at
         FROM llm_request_profiles WHERE id=$1",
    ).bind(id).fetch_optional(state.database()?).await.map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found("profile not found"))?;
    Ok(Json(row))
}

async fn update_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<LlmRequestProfileRow>, ApiError> {
    let existing = get_profile(State(state.clone()), Path(id)).await?;
    let name = match payload.name { Some(n) => { validate_name(&n)?; n } None => existing.0.name };
    let provider_id = match payload.provider_id { Some(p) => p, None => existing.0.provider_id };
    let ep = payload.endpoint_path.unwrap_or(existing.0.endpoint_path);
    let method = payload.method.unwrap_or(existing.0.method);
    let bb = payload.base_body.unwrap_or(existing.0.base_body);
    let headers = payload.headers.unwrap_or(existing.0.headers);
    let mim = payload.message_injection_mode.unwrap_or(existing.0.message_injection_mode);
    let enabled = payload.enabled.unwrap_or(existing.0.enabled);

    let row = sqlx::query_as::<_, LlmRequestProfileRow>(
        "UPDATE llm_request_profiles SET name=$1, provider_id=$2, endpoint_path=$3, method=$4, base_body=$5, headers=$6, message_injection_mode=$7, enabled=$8, updated_at=now()
         WHERE id=$9 RETURNING id, name, provider_id, endpoint_path, method, base_body, headers, message_injection_mode, enabled, created_at, updated_at",
    )
    .bind(&name).bind(provider_id).bind(&ep).bind(&method).bind(&bb).bind(&headers).bind(&mim).bind(enabled).bind(id)
    .fetch_one(state.database()?)
    .await
    .map_err(ApiError::from)?;
    Ok(Json(row))
}

async fn delete_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    let r = sqlx::query("DELETE FROM llm_request_profiles WHERE id=$1").bind(id).execute(state.database()?).await.map_err(ApiError::from)?;
    if r.rows_affected() == 0 { return Err(ApiError::not_found("not found")); }
    Ok((axum::http::StatusCode::OK, Json(serde_json::json!({"deleted":true}))))
}

async fn test_profile(
    State(state): State<AppState>,
    Path(profile_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let pool = state.database()?.clone();

    let profile = sqlx::query_as::<_, LlmRequestProfileRow>(
        "SELECT * FROM llm_request_profiles WHERE id=$1",
    ).bind(profile_id).fetch_optional(&pool).await.map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found("profile not found"))?;

    let provider = match profile.provider_id {
        Some(pid) => Some(
            sqlx::query_as::<_, LlmProviderRow>("SELECT * FROM llm_providers WHERE id=$1")
                .bind(pid).fetch_optional(&pool).await.map_err(ApiError::from)?
                .ok_or_else(|| ApiError::not_found("provider not found"))?,
        ),
        None => None,
    };

    let mut body = profile.base_body.clone();
    if profile.message_injection_mode == "replace_messages" {
        body["messages"] = serde_json::json!([{
            "role": "user",
            "content": "Hello, this is a test message."
        }]);
    }

    let headers_obj = profile.headers.clone();
    let url = if let Some(ref p) = provider {
        format!("{}{}", p.base_url.trim_end_matches('/'), profile.endpoint_path)
    } else {
        return Err(ApiError::validation("No provider configured"));
    };

    let client = reqwest::Client::new();
    let mut req = match profile.method.to_uppercase().as_str() {
        "POST" => client.post(&url),
        "GET" => client.get(&url),
        _ => client.post(&url),
    };

    req = req.header("Content-Type", "application/json");
    if let Some(ref p) = provider {
        if let Some(ref key) = p.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }
    if let Some(h) = headers_obj.as_object() {
        for (k, v) in h {
            if let Some(val) = v.as_str() {
                req = req.header(k.as_str(), val);
            }
        }
    }

    let started_at = Utc::now();
    let request_json = body.clone();

    let response = req.json(&body).send().await;
    let finished_at = Utc::now();

    match response {
        Ok(resp) => {
            let status_code = resp.status().as_u16();
            let resp_text = resp.text().await.unwrap_or_default();
            let resp_json: Value = serde_json::from_str(&resp_text).unwrap_or_else(|_| {
                serde_json::json!({"raw": resp_text})
            });

            let _ = sqlx::query(
                "INSERT INTO message_llm_calls (request_profile_id, provider_id, request_body, response_body, status, started_at, finished_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(profile_id).bind(profile.provider_id).bind(&request_json)
            .bind(&resp_json).bind(format!("{}", status_code))
            .bind(started_at).bind(finished_at)
            .execute(&pool).await;

            Ok(Json(serde_json::json!({
                "status": status_code,
                "success": status_code >= 200 && status_code < 300,
                "request": request_json,
                "response": resp_json,
            })))
        }
        Err(e) => {
            let _ = sqlx::query(
                "INSERT INTO message_llm_calls (request_profile_id, provider_id, request_body, status, error, started_at, finished_at)
                 VALUES ($1, $2, $3, 'error', $4, $5, $6)",
            )
            .bind(profile_id).bind(profile.provider_id).bind(&request_json)
            .bind(e.to_string()).bind(started_at).bind(finished_at)
            .execute(&pool).await;

            Ok(Json(serde_json::json!({
                "status": "error",
                "error": e.to_string(),
                "request": request_json,
            })))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::{json, Value};
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
    async fn provider_crud() {
        let Some(pool) = test_pool().await else { eprintln!("skip"); return; };
        let app = build_router(AppState { config: std::sync::Arc::new(AppConfig::for_test()), db: Some(pool.clone()) });

        let name = unique("p");
        let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/llm/providers")
            .header("content-type","application/json")
            .body(Body::from(json!({"name":name,"base_url":"https://api.openai.com/v1"}).to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let p: Value = serde_json::from_slice(&body).unwrap();
        let id = p["id"].as_str().unwrap();

        let resp = app.clone().oneshot(Request::builder().uri(format!("/api/llm/providers/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app.clone().oneshot(Request::builder().method("DELETE").uri(format!("/api/llm/providers/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn profile_crud() {
        let Some(pool) = test_pool().await else { eprintln!("skip"); return; };
        let app = build_router(AppState { config: std::sync::Arc::new(AppConfig::for_test()), db: Some(pool.clone()) });

        let name = unique("prof");
        let body = json!({"name":name,"base_body":{"model":"gpt-4","temperature":0.7},"headers":{"X-Custom":"test"}});
        let resp = app.clone().oneshot(Request::builder().method("POST").uri("/api/llm/request-profiles")
            .header("content-type","application/json")
            .body(Body::from(body.to_string())).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let b = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let p: Value = serde_json::from_slice(&b).unwrap();
        let id = p["id"].as_str().unwrap();

        let resp = app.clone().oneshot(Request::builder().uri(format!("/api/llm/request-profiles/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let resp = app.clone().oneshot(Request::builder().method("DELETE").uri(format!("/api/llm/request-profiles/{id}")).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
