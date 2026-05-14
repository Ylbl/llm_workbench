use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::convert::Infallible;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{ApiError, AppState};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ConversationRow {
    pub id: Uuid,
    pub workspace_item_id: Uuid,
    pub title: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub role: String,
    pub content: String,
    pub content_json: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MessageEventRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub message_id: Option<Uuid>,
    pub agent_run_id: Option<Uuid>,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub title: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateConversationRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppendMessageRequest {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub content_json: Option<Value>,
    #[serde(default)]
    pub parent_message_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<MessageRow>,
}

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<MessageEventRow>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub conversation_id: Uuid,
    #[serde(default)]
    pub agent_run_id: Option<Uuid>,
}

#[derive(Clone)]
pub struct ConversationService {
    pool: PgPool,
}

impl ConversationService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<ConversationRow>, ApiError> {
        list_conversations_from_pool(&self.pool).await
    }

    pub async fn create(
        &self,
        req: CreateConversationRequest,
    ) -> Result<ConversationRow, ApiError> {
        create_conversation_in_pool(&self.pool, req).await
    }

    pub async fn get(&self, id: Uuid) -> Result<ConversationRow, ApiError> {
        get_conversation_from_pool(&self.pool, id).await
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: UpdateConversationRequest,
    ) -> Result<ConversationRow, ApiError> {
        update_conversation_in_pool(&self.pool, id, req).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        delete_conversation_in_pool(&self.pool, id).await
    }

    pub async fn list_messages(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<MessageRow>, ApiError> {
        list_messages_from_pool(&self.pool, conversation_id).await
    }

    pub async fn append_message(
        &self,
        conversation_id: Uuid,
        req: AppendMessageRequest,
    ) -> Result<MessageRow, ApiError> {
        append_message_in_pool(&self.pool, conversation_id, req).await
    }

    pub async fn list_events(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<MessageEventRow>, ApiError> {
        list_events_from_pool(&self.pool, conversation_id).await
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/conversations",
            get(list_conversations).post(create_conversation),
        )
        .route(
            "/api/conversations/{id}",
            get(get_conversation).patch(update_conversation).delete(delete_conversation),
        )
        .route(
            "/api/conversations/{id}/messages",
            get(list_conversation_messages).post(append_conversation_message),
        )
        .route(
            "/api/conversations/{id}/events",
            get(list_conversation_events),
        )
        .route(
            "/api/events",
            get(sse_events),
        )
        .route(
            "/api/conversations/{id}/stream",
            post(stream_conversation),
        )
}

async fn list_conversations(
    State(state): State<AppState>,
) -> Result<Json<Vec<ConversationRow>>, ApiError> {
    let items = list_conversations_from_pool(state.database()?).await?;
    Ok(Json(items))
}

async fn create_conversation(
    State(state): State<AppState>,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<(axum::http::StatusCode, Json<ConversationRow>), ApiError> {
    let conv = create_conversation_in_pool(state.database()?, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(conv)))
}

async fn get_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ConversationRow>, ApiError> {
    let conv = get_conversation_from_pool(state.database()?, id).await?;
    Ok(Json(conv))
}

async fn update_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateConversationRequest>,
) -> Result<Json<ConversationRow>, ApiError> {
    let conv = update_conversation_in_pool(state.database()?, id, payload).await?;
    Ok(Json(conv))
}

async fn delete_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    delete_conversation_in_pool(state.database()?, id).await?;
    Ok((
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "deleted": true })),
    ))
}

async fn list_conversation_messages(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<MessagesResponse>, ApiError> {
    let messages = list_messages_from_pool(state.database()?, id).await?;
    Ok(Json(MessagesResponse { messages }))
}

async fn append_conversation_message(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AppendMessageRequest>,
) -> Result<(axum::http::StatusCode, Json<MessageRow>), ApiError> {
    let msg = append_message_in_pool(state.database()?, id, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(msg)))
}

async fn list_conversation_events(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<EventsResponse>, ApiError> {
    let events = list_events_from_pool(state.database()?, id).await?;
    Ok(Json(EventsResponse { events }))
}

fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::validation("title must not be empty"));
    }
    Ok(())
}

fn validate_role(role: &str) -> Result<(), ApiError> {
    match role {
        "user" | "assistant" | "system" | "tool" => Ok(()),
        other => Err(ApiError::validation(format!(
            "Invalid role '{}'. Must be user, assistant, system, or tool",
            other
        ))),
    }
}

async fn list_conversations_from_pool(
    pool: &PgPool,
) -> Result<Vec<ConversationRow>, ApiError> {
    let items = sqlx::query_as::<_, ConversationRow>(
        "SELECT id, workspace_item_id, title, status, created_at, updated_at
         FROM conversations
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(items)
}

async fn create_conversation_in_pool(
    pool: &PgPool,
    req: CreateConversationRequest,
) -> Result<ConversationRow, ApiError> {
    validate_title(&req.title)?;

    let workspace_item_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO workspace_items (item_type, title, parent_id, sort_order, metadata)
         VALUES ('chat', $1, $2, 0, '{}')
         RETURNING id",
    )
    .bind(req.title.trim())
    .bind(req.parent_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    let conv = sqlx::query_as::<_, ConversationRow>(
        "INSERT INTO conversations (workspace_item_id, title)
         VALUES ($1, $2)
         RETURNING id, workspace_item_id, title, status, created_at, updated_at",
    )
    .bind(workspace_item_id)
    .bind(req.title.trim())
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(conv)
}

async fn get_conversation_from_pool(
    pool: &PgPool,
    id: Uuid,
) -> Result<ConversationRow, ApiError> {
    let conv = sqlx::query_as::<_, ConversationRow>(
        "SELECT id, workspace_item_id, title, status, created_at, updated_at
         FROM conversations WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Conversation {id} not found")))?;

    Ok(conv)
}

async fn update_conversation_in_pool(
    pool: &PgPool,
    id: Uuid,
    req: UpdateConversationRequest,
) -> Result<ConversationRow, ApiError> {
    let existing = get_conversation_from_pool(pool, id).await?;

    let title = match req.title {
        Some(ref t) => {
            validate_title(t)?;
            t.trim().to_string()
        }
        None => existing.title,
    };

    let status = req.status.unwrap_or(existing.status);

    let conv = sqlx::query_as::<_, ConversationRow>(
        "UPDATE conversations SET title = $1, status = $2, updated_at = now()
         WHERE id = $3
         RETURNING id, workspace_item_id, title, status, created_at, updated_at",
    )
    .bind(&title)
    .bind(&status)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    sqlx::query(
        "UPDATE workspace_items SET title = $1, updated_at = now() WHERE id = $2",
    )
    .bind(&title)
    .bind(existing.workspace_item_id)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(conv)
}

async fn delete_conversation_in_pool(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
    let result = sqlx::query("DELETE FROM conversations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(format!("Conversation {id} not found")));
    }

    Ok(())
}

async fn list_messages_from_pool(
    pool: &PgPool,
    conversation_id: Uuid,
) -> Result<Vec<MessageRow>, ApiError> {
    let messages = sqlx::query_as::<_, MessageRow>(
        "SELECT id, conversation_id, parent_message_id, role, content, content_json, created_at
         FROM messages
         WHERE conversation_id = $1
         ORDER BY created_at",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(messages)
}

async fn append_message_in_pool(
    pool: &PgPool,
    conversation_id: Uuid,
    req: AppendMessageRequest,
) -> Result<MessageRow, ApiError> {
    validate_role(&req.role)?;

    let msg = sqlx::query_as::<_, MessageRow>(
        "INSERT INTO messages (conversation_id, parent_message_id, role, content, content_json)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, conversation_id, parent_message_id, role, content, content_json, created_at",
    )
    .bind(conversation_id)
    .bind(req.parent_message_id)
    .bind(&req.role)
    .bind(&req.content)
    .bind(&req.content_json)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    sqlx::query(
        "UPDATE conversations SET updated_at = now() WHERE id = $1",
    )
    .bind(conversation_id)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(msg)
}

#[derive(Debug, Deserialize)]
struct StreamRequest {
    request_profile_id: Uuid,
    #[serde(default)]
    prompt_block_ids: Vec<Uuid>,
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default)]
    runtime_overrides: Option<Value>,
    #[serde(default)]
    raw_body_overrides: Option<Value>,
}

async fn stream_conversation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<StreamRequest>,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let pool = state.database()?.clone();
    let conv_id = id;
    let req_profile_id = payload.request_profile_id;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(128);
    let pool_clone = pool.clone();

    tokio::spawn(async move {
        let result = crate::llm::stream_llm_response(
            &pool_clone, conv_id, req_profile_id, &payload.prompt_block_ids,
            payload.system_prompt.as_deref(), None, tx.clone(),
        ).await;

        match result {
            Ok((full_content, request_body)) => {
                let msg = sqlx::query_as::<_, MessageRow>(
                    "INSERT INTO messages (conversation_id, role, content)
                     VALUES ($1, 'assistant', $2)
                     RETURNING id, conversation_id, parent_message_id, role, content, content_json, created_at",
                )
                .bind(conv_id)
                .bind(&full_content)
                .fetch_one(&pool_clone)
                .await;

                if let Ok(ref msg) = msg {
                    let _ = sqlx::query(
                        "INSERT INTO message_events (conversation_id, message_id, event_type, payload)
                         VALUES ($1, $2, 'llm.done', '{}')",
                    )
                    .bind(conv_id).bind(msg.id)
                    .execute(&pool_clone).await;

                    let _ = sqlx::query(
                        "INSERT INTO message_llm_calls (conversation_id, message_id, request_profile_id, provider_id, request_body, status, started_at, finished_at)
                         VALUES ($1, $2, $3, (SELECT provider_id FROM llm_request_profiles WHERE id = $3), $4, 'completed', now(), now())",
                    )
                    .bind(conv_id).bind(msg.id).bind(req_profile_id).bind(&request_body)
                    .execute(&pool_clone).await;

                    let payload = serde_json::json!({
                        "conversation_id": conv_id,
                        "message_id": msg.id,
                    });
                    let _ = tx.send(Ok(Event::default().event("llm.done").json_data(payload).unwrap())).await;
                }
            }
            Err(e) => {
                let _ = tx.send(Ok(Event::default()
                    .event("llm.error")
                    .json_data(serde_json::json!({"error": e.to_string()})).unwrap())).await;
            }
        }

        let _ = sqlx::query("UPDATE conversations SET updated_at = now() WHERE id = $1")
            .bind(conv_id).execute(&pool_clone).await;
    });

    Ok(Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default()))
}

async fn append_mock_assistant(
    pool: &PgPool,
    conversation_id: Uuid,
) -> Result<MessageRow, ApiError> {
    let content = concat!(
        "This is a mock assistant response.\n\n",
        "Here is an inline formula: $E = mc^2$\n\n",
        "And a quadratic formula inline: \\(x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\\)\n\n",
        "A block formula:\n\n",
        "$$\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}$$\n\n",
        "Another block with brackets:\n\n",
        "\\[\\sum_{n=1}^{\\infty} \\frac{1}{n^2} = \\frac{\\pi^2}{6}\\]\n\n",
        "Here is some streaming-like incomplete text: $E = mc (no closing dollar sign, should stay raw)\n\n",
        "Real LLM integration will come in a later batch."
    );

    let msg = sqlx::query_as::<_, MessageRow>(
        "INSERT INTO messages (conversation_id, role, content)
         VALUES ($1, 'assistant', $2)
         RETURNING id, conversation_id, parent_message_id, role, content, content_json, created_at",
    )
    .bind(conversation_id)
    .bind(content)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    sqlx::query(
        "INSERT INTO message_events (conversation_id, message_id, event_type, payload)
         VALUES ($1, $2, 'mock.response', '{}')",
    )
    .bind(conversation_id)
    .bind(msg.id)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(msg)
}

async fn list_events_from_pool(
    pool: &PgPool,
    conversation_id: Uuid,
) -> Result<Vec<MessageEventRow>, ApiError> {
    let events = sqlx::query_as::<_, MessageEventRow>(
        "SELECT id, conversation_id, message_id, agent_run_id, event_type, payload, created_at
         FROM message_events
         WHERE conversation_id = $1
         ORDER BY created_at",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(events)
}

async fn sse_events(
    State(state): State<AppState>,
    Query(query): Query<EventsQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(64);
    let pool_opt = state.db.clone();
    let conv_id = query.conversation_id;

    tokio::spawn(async move {
        if let Some(pool) = pool_opt {
            mock_stream(pool, conv_id, tx).await;
        }
    });

    Sse::new(ReceiverStream::new(rx)).keep_alive(KeepAlive::default())
}

async fn mock_stream(
    pool: PgPool,
    conversation_id: Uuid,
    tx: mpsc::Sender<Result<Event, Infallible>>,
) {
    let chunks: Vec<&str> = vec![
        "This ", "is ", "a ", "mock ", "streaming ", "response.\n\n",
        "Here ", "is ", "an ", "inline ", "formula: ", "$E=mc^2$\n\n",
        "And ", "a ", "quadratic: ", "\\(x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}\\)\n\n",
        "A ", "block ", "formula:\n\n",
        "$$\\int_{-\\infty}^{\\infty} e^{-x^2} dx = \\sqrt{\\pi}$$\n\n",
        "Streaming ", "test: ", "$E = mc",
    ];

    for chunk in &chunks {
        let payload = serde_json::json!({
            "conversation_id": conversation_id,
            "delta": chunk,
        });
        let _ = tx
            .send(Ok(Event::default().event("llm.delta").json_data(payload).unwrap()))
            .await;
        tokio::time::sleep(Duration::from_millis(80)).await;
    }

    let full_content: String = chunks.iter().copied().collect();

    let msg = sqlx::query_as::<_, MessageRow>(
        "INSERT INTO messages (conversation_id, role, content)
         VALUES ($1, 'assistant', $2)
         RETURNING id, conversation_id, parent_message_id, role, content, content_json, created_at",
    )
    .bind(conversation_id)
    .bind(&full_content)
    .fetch_one(&pool)
    .await;

    if let Ok(msg) = msg {
        let _ = sqlx::query(
            "INSERT INTO message_events (conversation_id, message_id, event_type, payload)
             VALUES ($1, $2, 'llm.done', '{}')",
        )
        .bind(conversation_id)
        .bind(msg.id)
        .execute(&pool)
        .await;

        let payload = serde_json::json!({
            "conversation_id": conversation_id,
            "message_id": msg.id,
        });
        let _ = tx
            .send(Ok(Event::default().event("llm.done").json_data(payload).unwrap()))
            .await;
    }

    let _ = sqlx::query(
        "UPDATE conversations SET updated_at = now() WHERE id = $1",
    )
    .bind(conversation_id)
    .execute(&pool)
    .await;
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

    fn unique_id(prefix: &str) -> String {
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

    #[tokio::test]
    async fn conversations_endpoint_reports_database_error_without_pool() {
        let app = build_router(AppState::new_without_db(AppConfig::for_test()));

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/conversations")
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
    async fn conversation_crud_when_database_is_available() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping PostgreSQL conversation test; TEST_DATABASE_URL is not set");
            return;
        };

        let app = build_router(AppState {
            config: std::sync::Arc::new(AppConfig::for_test()),
            db: Some(pool.clone()),
        });

        let title = unique_id("conv.test");

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/conversations")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "title": title }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let conv_id = payload["id"].as_str().unwrap().to_string();

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/conversations/{conv_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/conversations/{conv_id}/messages"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({ "role": "user", "content": "Hello" }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/api/conversations/{conv_id}/messages"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: Value = serde_json::from_slice(&body).unwrap();
        let messages = payload["messages"].as_array().unwrap();
        assert!(messages.len() >= 1);
        let roles: Vec<&str> = messages
            .iter()
            .map(|m| m["role"].as_str().unwrap())
            .collect();
        assert!(roles.contains(&"user"));

        // Also test mock_stream via the service layer
        let service = ConversationService::new(pool.clone());
        let result = append_mock_assistant(&pool, Uuid::parse_str(&conv_id).unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sse_stream_endpoint_returns_ok() {
        let Some(pool) = test_pool().await else {
            eprintln!("skipping SSE test; TEST_DATABASE_URL is not set");
            return;
        };

        let title = unique_id("sse.test");
        let service = ConversationService::new(pool.clone());
        let conv = service
            .create(CreateConversationRequest {
                title: title.clone(),
                parent_id: None,
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
                    .uri(format!("/api/events?conversation_id={}", conv.id))
                    .header("accept", "text/event-stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());
    }
}
