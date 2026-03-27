pub mod db;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageCreate {
    pub sender: String,
    pub recipient: String,
    pub content: String,
    #[serde(default)]
    pub refs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub hash: String,
    pub sender: String,
    pub recipient: String,
    pub content: String,
    pub refs: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub instance_id: String,
    pub last_seen: DateTime<Utc>,
    pub message_count: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/messages", post(create_message))
        .route("/messages/:instance_id", get(list_messages))
        .route("/history/:instance_id", get(get_history))
        .route("/roster", get(get_roster))
        .route("/messages/cleanup", delete(cleanup_old_messages))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state))
}

#[cfg(test)]
pub async fn create_test_app() -> Router {
    let db = db::init_test_db().await.unwrap();
    let state = AppState { db };
    create_app(state)
}

async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "Claude IPC Server",
        "version": "0.1.0"
    }))
}

async fn list_messages(
    Path(instance_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // Calculate cutoff time (1 hour ago)
    let one_hour_ago = Utc::now() - Duration::hours(1);
    let cutoff_iso = one_hour_ago.to_rfc3339();

    let rows = sqlx::query(
        r#"
        SELECT id, hash, sender, recipient, content, refs, timestamp
        FROM messages
        WHERE recipient = ? AND timestamp >= ?
        ORDER BY timestamp DESC
        "#,
    )
    .bind(&instance_id)
    .bind(&cutoff_iso)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let messages: Vec<Message> = rows
        .into_iter()
        .filter_map(|row| {
            let refs_str: String = row.get("refs");
            let refs = if refs_str.is_empty() {
                vec![]
            } else {
                refs_str.split(',').map(|s| s.to_string()).collect()
            };

            let timestamp_str: String = row.get("timestamp");
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .ok()?
                .with_timezone(&Utc);

            Some(Message {
                id: row.get("id"),
                hash: row.get("hash"),
                sender: row.get("sender"),
                recipient: row.get("recipient"),
                content: row.get("content"),
                refs,
                timestamp,
            })
        })
        .collect();

    Ok(Json(messages))
}

async fn get_history(
    Path(instance_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Message>>, StatusCode> {
    // Get all messages (sent or received) for this instance from the last hour
    let one_hour_ago = Utc::now() - Duration::hours(1);
    let cutoff_iso = one_hour_ago.to_rfc3339();

    let rows = sqlx::query(
        r#"
        SELECT id, hash, sender, recipient, content, refs, timestamp
        FROM messages
        WHERE (recipient = ? OR sender = ?) AND timestamp >= ?
        ORDER BY timestamp ASC
        "#,
    )
    .bind(&instance_id)
    .bind(&instance_id)
    .bind(&cutoff_iso)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let messages: Vec<Message> = rows
        .into_iter()
        .filter_map(|row| {
            let refs_str: String = row.get("refs");
            let refs = if refs_str.is_empty() {
                vec![]
            } else {
                refs_str.split(',').map(|s| s.to_string()).collect()
            };

            let timestamp_str: String = row.get("timestamp");
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .ok()?
                .with_timezone(&Utc);

            Some(Message {
                id: row.get("id"),
                hash: row.get("hash"),
                sender: row.get("sender"),
                recipient: row.get("recipient"),
                content: row.get("content"),
                refs,
                timestamp,
            })
        })
        .collect();

    Ok(Json(messages))
}

async fn get_roster(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkerInfo>>, StatusCode> {
    // Get all unique workers who have sent messages in the last hour
    let one_hour_ago = Utc::now() - Duration::hours(1);
    let cutoff_iso = one_hour_ago.to_rfc3339();

    let rows = sqlx::query(
        r#"
        SELECT sender as instance_id, MAX(timestamp) as last_seen, COUNT(*) as message_count
        FROM messages
        WHERE timestamp >= ?
        GROUP BY sender
        ORDER BY last_seen DESC
        "#,
    )
    .bind(&cutoff_iso)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let workers: Vec<WorkerInfo> = rows
        .into_iter()
        .filter_map(|row| {
            let timestamp_str: String = row.get("last_seen");
            let last_seen = DateTime::parse_from_rfc3339(&timestamp_str)
                .ok()?
                .with_timezone(&Utc);

            Some(WorkerInfo {
                instance_id: row.get("instance_id"),
                last_seen,
                message_count: row.get::<i64, _>("message_count") as usize,
            })
        })
        .collect();

    Ok(Json(workers))
}

async fn create_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MessageCreate>,
) -> Result<Json<Message>, StatusCode> {
    // Generate SHA1 hash of content
    let mut hasher = Sha1::new();
    hasher.update(payload.content.as_bytes());
    let content_hash = format!("{:x}", hasher.finalize());

    // Generate unique ID
    let message_id = Uuid::new_v4().to_string();

    // Current timestamp
    let timestamp = Utc::now();
    let timestamp_iso = timestamp.to_rfc3339();

    // Store refs as comma-separated string
    let refs_str = payload.refs.join(",");

    sqlx::query(
        r#"
        INSERT INTO messages (id, hash, sender, recipient, content, refs, timestamp)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&message_id)
    .bind(&content_hash)
    .bind(&payload.sender)
    .bind(&payload.recipient)
    .bind(&payload.content)
    .bind(&refs_str)
    .bind(&timestamp_iso)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(Message {
        id: message_id,
        hash: content_hash,
        sender: payload.sender,
        recipient: payload.recipient,
        content: payload.content,
        refs: payload.refs,
        timestamp,
    }))
}

async fn cleanup_old_messages(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let one_hour_ago = Utc::now() - Duration::hours(1);
    let cutoff_iso = one_hour_ago.to_rfc3339();

    let result = sqlx::query(
        r#"
        DELETE FROM messages WHERE timestamp < ?
        "#,
    )
    .bind(&cutoff_iso)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Database error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "deleted": result.rows_affected()
    })))
}
