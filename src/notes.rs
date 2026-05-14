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
pub struct NoteRow {
    pub id: Uuid,
    pub workspace_item_id: Uuid,
    pub title: String,
    pub document_json: Value,
    pub plain_text: String,
    pub format: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct NoteRevisionRow {
    pub id: Uuid,
    pub note_id: Uuid,
    pub document_json: Value,
    pub plain_text: String,
    pub reason: Option<String>,
    pub created_by: String,
    pub agent_run_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    #[serde(default)]
    pub workspace_item_id: Option<Uuid>,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default = "default_sort_order")]
    pub sort_order: i32,
    #[serde(default = "default_document")]
    pub document_json: Option<Value>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

fn default_sort_order() -> i32 {
    0
}

fn default_document() -> Option<Value> {
    None
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub document_json: Option<Value>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct NoteRevisionListResponse {
    pub revisions: Vec<NoteRevisionRow>,
}

#[derive(Clone)]
pub struct NoteService {
    pool: PgPool,
}

impl NoteService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> Result<Vec<NoteRow>, ApiError> {
        list_notes_from_pool(&self.pool).await
    }

    pub async fn create(&self, req: CreateNoteRequest) -> Result<NoteRow, ApiError> {
        create_note_in_pool(&self.pool, req).await
    }

    pub async fn get(&self, id: Uuid) -> Result<NoteRow, ApiError> {
        get_note_from_pool(&self.pool, id).await
    }

    pub async fn get_by_workspace_item(
        &self,
        workspace_item_id: Uuid,
    ) -> Result<NoteRow, ApiError> {
        get_note_by_workspace_item_from_pool(&self.pool, workspace_item_id).await
    }

    pub async fn update(&self, id: Uuid, req: UpdateNoteRequest) -> Result<NoteRow, ApiError> {
        update_note_in_pool(&self.pool, id, req).await
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ApiError> {
        delete_note_in_pool(&self.pool, id).await
    }

    pub async fn list_revisions(
        &self,
        note_id: Uuid,
    ) -> Result<Vec<NoteRevisionRow>, ApiError> {
        list_revisions_from_pool(&self.pool, note_id).await
    }

    pub async fn restore_revision(
        &self,
        note_id: Uuid,
        revision_id: Uuid,
    ) -> Result<NoteRow, ApiError> {
        restore_revision_in_pool(&self.pool, note_id, revision_id).await
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/notes", get(list_notes).post(create_note))
        .route(
            "/api/notes/{id}",
            get(get_note).patch(update_note).delete(delete_note),
        )
        .route(
            "/api/notes/workspace/{workspace_item_id}",
            get(get_note_by_workspace_item),
        )
        .route(
            "/api/notes/{id}/revisions",
            get(list_note_revisions),
        )
        .route(
            "/api/notes/{id}/revisions/{revision_id}/restore",
            get(restore_note_revision).post(restore_note_revision),
        )
}

async fn list_notes(
    State(state): State<AppState>,
) -> Result<Json<Vec<NoteRow>>, ApiError> {
    let notes = list_notes_from_pool(state.database()?).await?;
    Ok(Json(notes))
}

async fn create_note(
    State(state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<(axum::http::StatusCode, Json<NoteRow>), ApiError> {
    let note = create_note_in_pool(state.database()?, payload).await?;
    Ok((axum::http::StatusCode::CREATED, Json(note)))
}

async fn get_note(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NoteRow>, ApiError> {
    let note = get_note_from_pool(state.database()?, id).await?;
    Ok(Json(note))
}

async fn get_note_by_workspace_item(
    State(state): State<AppState>,
    Path(workspace_item_id): Path<Uuid>,
) -> Result<Json<NoteRow>, ApiError> {
    let note = get_note_by_workspace_item_from_pool(state.database()?, workspace_item_id).await?;
    Ok(Json(note))
}

async fn update_note(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<NoteRow>, ApiError> {
    let note = update_note_in_pool(state.database()?, id, payload).await?;
    Ok(Json(note))
}

async fn delete_note(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(axum::http::StatusCode, Json<Value>), ApiError> {
    delete_note_in_pool(state.database()?, id).await?;
    Ok((
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "deleted": true })),
    ))
}

async fn list_note_revisions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<NoteRevisionListResponse>, ApiError> {
    let revisions = list_revisions_from_pool(state.database()?, id).await?;
    Ok(Json(NoteRevisionListResponse { revisions }))
}

async fn restore_note_revision(
    State(state): State<AppState>,
    Path((id, revision_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<NoteRow>, ApiError> {
    let note = restore_revision_in_pool(state.database()?, id, revision_id).await?;
    Ok(Json(note))
}

fn extract_plain_text(doc: &Value) -> String {
    let Some(content) = doc.get("content").and_then(|c| c.as_array()) else {
        return String::new();
    };

    let mut text = String::new();
    extract_text_from_nodes(content, &mut text, true);
    text.trim().to_string()
}

fn extract_text_from_nodes(nodes: &[Value], text: &mut String, first: bool) {
    for node in nodes {
        let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match node_type {
            "text" => {
                if let Some(t) = node.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            }
            "hardBreak" => {
                text.push('\n');
            }
            "paragraph" => {
                if !first {
                    text.push('\n');
                }
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
                text.push('\n');
            }
            "heading" => {
                if !first {
                    text.push('\n');
                }
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
                text.push('\n');
            }
            "bulletList" | "orderedList" => {
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
            }
            "listItem" => {
                text.push_str("\n• ");
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
            }
            "blockquote" => {
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
            }
            "codeBlock" => {
                if !first {
                    text.push('\n');
                }
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
                text.push('\n');
            }
            "inlineMath" | "blockMath" => {
                if let Some(latex) = node.get("attrs")
                    .and_then(|a| a.get("latex"))
                    .and_then(|l| l.as_str())
                {
                    if node_type == "blockMath" {
                        text.push('\n');
                    }
                    text.push('$');
                    text.push_str(latex);
                    text.push('$');
                    if node_type == "blockMath" {
                        text.push('\n');
                    }
                }
            }
            _ => {
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    extract_text_from_nodes(content, text, false);
                }
            }
        }
    }
}

fn validate_title(title: &str) -> Result<(), ApiError> {
    if title.trim().is_empty() {
        return Err(ApiError::validation("title must not be empty"));
    }
    Ok(())
}

async fn list_notes_from_pool(pool: &PgPool) -> Result<Vec<NoteRow>, ApiError> {
    let notes = sqlx::query_as::<_, NoteRow>(
        "SELECT id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at
         FROM notes
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(notes)
}

async fn create_note_in_pool(
    pool: &PgPool,
    req: CreateNoteRequest,
) -> Result<NoteRow, ApiError> {
    validate_title(&req.title)?;

    let document_json = req.document_json.unwrap_or_else(|| {
        serde_json::json!({ "type": "doc", "content": [] })
    });
    let plain_text = extract_plain_text(&document_json);
    let metadata = req.metadata.unwrap_or(serde_json::Value::Object(Default::default()));

    let workspace_item_id = if let Some(wid) = req.workspace_item_id {
        wid
    } else {
        sqlx::query_scalar::<_, Uuid>(
            "INSERT INTO workspace_items (item_type, title, parent_id, sort_order, metadata)
             VALUES ('note', $1, $2, $3, '{}')
             RETURNING id",
        )
        .bind(req.title.trim())
        .bind(req.parent_id)
        .bind(req.sort_order)
        .fetch_one(pool)
        .await
        .map_err(ApiError::from)?
    };

    let note = sqlx::query_as::<_, NoteRow>(
        "INSERT INTO notes (workspace_item_id, title, document_json, plain_text, format, metadata)
         VALUES ($1, $2, $3, $4, 'tiptap_json', $5)
         RETURNING id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at",
    )
    .bind(workspace_item_id)
    .bind(req.title.trim())
    .bind(&document_json)
    .bind(&plain_text)
    .bind(&metadata)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(note)
}

async fn get_note_from_pool(pool: &PgPool, id: Uuid) -> Result<NoteRow, ApiError> {
    let note = sqlx::query_as::<_, NoteRow>(
        "SELECT id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at
         FROM notes WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Note {id} not found")))?;

    Ok(note)
}

async fn get_note_by_workspace_item_from_pool(
    pool: &PgPool,
    workspace_item_id: Uuid,
) -> Result<NoteRow, ApiError> {
    let note = sqlx::query_as::<_, NoteRow>(
        "SELECT id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at
         FROM notes WHERE workspace_item_id = $1",
    )
    .bind(workspace_item_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!(
        "Note for workspace item {workspace_item_id} not found"
    )))?;

    Ok(note)
}

async fn update_note_in_pool(
    pool: &PgPool,
    id: Uuid,
    req: UpdateNoteRequest,
) -> Result<NoteRow, ApiError> {
    let existing = get_note_from_pool(pool, id).await?;

    let title = match req.title {
        Some(ref t) => {
            validate_title(t)?;
            t.trim().to_string()
        }
        None => existing.title,
    };

    let metadata = req.metadata.unwrap_or(existing.metadata);

    if let Some(ref doc) = req.document_json {
        let plain_text = extract_plain_text(doc);

        sqlx::query(
            "INSERT INTO note_revisions (note_id, document_json, plain_text, reason, created_by)
             VALUES ($1, $2, $3, 'update', 'user')",
        )
        .bind(id)
        .bind(&existing.document_json)
        .bind(&existing.plain_text)
        .execute(pool)
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

        let note = sqlx::query_as::<_, NoteRow>(
            "UPDATE notes SET title = $1, document_json = $2, plain_text = $3, metadata = $4, updated_at = now()
             WHERE id = $5
             RETURNING id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at",
        )
        .bind(&title)
        .bind(doc)
        .bind(&plain_text)
        .bind(&metadata)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(ApiError::from)?;

        return Ok(note);
    }

    sqlx::query(
        "UPDATE workspace_items SET title = $1, updated_at = now() WHERE id = $2",
    )
    .bind(&title)
    .bind(existing.workspace_item_id)
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    let note = sqlx::query_as::<_, NoteRow>(
        "UPDATE notes SET title = $1, metadata = $2, updated_at = now()
         WHERE id = $3
         RETURNING id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at",
    )
    .bind(&title)
    .bind(&metadata)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(note)
}

async fn delete_note_in_pool(pool: &PgPool, id: Uuid) -> Result<(), ApiError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(ApiError::from)?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(format!("Note {id} not found")));
    }

    Ok(())
}

async fn list_revisions_from_pool(
    pool: &PgPool,
    note_id: Uuid,
) -> Result<Vec<NoteRevisionRow>, ApiError> {
    let revisions = sqlx::query_as::<_, NoteRevisionRow>(
        "SELECT id, note_id, document_json, plain_text, reason, created_by, agent_run_id, created_at
         FROM note_revisions
         WHERE note_id = $1
         ORDER BY created_at DESC",
    )
    .bind(note_id)
    .fetch_all(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(revisions)
}

async fn restore_revision_in_pool(
    pool: &PgPool,
    note_id: Uuid,
    revision_id: Uuid,
) -> Result<NoteRow, ApiError> {
    let existing = get_note_from_pool(pool, note_id).await?;

    let revision = sqlx::query_as::<_, NoteRevisionRow>(
        "SELECT id, note_id, document_json, plain_text, reason, created_by, agent_run_id, created_at
         FROM note_revisions
         WHERE id = $1 AND note_id = $2",
    )
    .bind(revision_id)
    .bind(note_id)
    .fetch_optional(pool)
    .await
    .map_err(ApiError::from)?
    .ok_or_else(|| ApiError::not_found(format!("Revision {revision_id} not found")))?;

    sqlx::query(
        "INSERT INTO note_revisions (note_id, document_json, plain_text, reason, created_by)
         VALUES ($1, $2, $3, $4, 'user')",
    )
    .bind(note_id)
    .bind(&existing.document_json)
    .bind(&existing.plain_text)
    .bind(format!("restore to revision {}", revision_id))
    .execute(pool)
    .await
    .map_err(ApiError::from)?;

    let note = sqlx::query_as::<_, NoteRow>(
        "UPDATE notes SET document_json = $1, plain_text = $2, updated_at = now()
         WHERE id = $3
         RETURNING id, workspace_item_id, title, document_json, plain_text, format, metadata, created_at, updated_at",
    )
    .bind(&revision.document_json)
    .bind(&revision.plain_text)
    .bind(note_id)
    .fetch_one(pool)
    .await
    .map_err(ApiError::from)?;

    Ok(note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn plain_text_from_paragraph() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        { "type": "text", "text": "Hello world" }
                    ]
                }
            ]
        });
        assert_eq!(extract_plain_text(&doc), "Hello world");
    }

    #[test]
    fn plain_text_from_multiple_paragraphs() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "First line" }]
                },
                {
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "Second line" }]
                }
            ]
        });
        let plain = extract_plain_text(&doc);
        assert!(plain.contains("First line"));
        assert!(plain.contains("Second line"));
    }

    #[test]
    fn plain_text_from_heading() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": { "level": 1 },
                    "content": [{ "type": "text", "text": "Title" }]
                },
                {
                    "type": "paragraph",
                    "content": [{ "type": "text", "text": "Body" }]
                }
            ]
        });
        let plain = extract_plain_text(&doc);
        assert!(plain.contains("Title"));
        assert!(plain.contains("Body"));
    }

    #[test]
    fn plain_text_from_inline_math() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        { "type": "text", "text": "Formula: " },
                        { "type": "inlineMath", "attrs": { "latex": "E=mc^2" } }
                    ]
                }
            ]
        });
        let plain = extract_plain_text(&doc);
        assert!(plain.contains("$E=mc^2$"));
    }

    #[test]
    fn plain_text_from_block_math() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "blockMath",
                    "attrs": { "latex": "\\int_0^1 x dx" }
                }
            ]
        });
        let plain = extract_plain_text(&doc);
        assert!(plain.contains("$\\int_0^1 x dx$"));
    }

    #[test]
    fn plain_text_from_list() {
        let doc = json!({
            "type": "doc",
            "content": [
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{ "type": "text", "text": "Item 1" }]
                                }
                            ]
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{ "type": "text", "text": "Item 2" }]
                                }
                            ]
                        }
                    ]
                }
            ]
        });
        let plain = extract_plain_text(&doc);
        assert!(plain.contains("Item 1"));
        assert!(plain.contains("Item 2"));
    }

    #[test]
    fn plain_text_empty_doc() {
        let doc = json!({ "type": "doc", "content": [] });
        assert_eq!(extract_plain_text(&doc), "");
    }
}
