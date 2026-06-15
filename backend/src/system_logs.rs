use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{types::Json as SqlJson, FromRow};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_system_logs).post(create_system_log))
        .route("/:id", get(get_system_log))
}

#[derive(Debug, Deserialize)]
struct SystemLogQuery {
    service_name: Option<String>,
    severity: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SystemLogCreate {
    service_name: String,
    severity: String,
    message: String,
    context: Option<Value>,
}

#[derive(Debug, FromRow, Serialize)]
struct SystemLogRow {
    id: i64,
    service_name: String,
    severity: String,
    message: String,
    context: Option<Value>,
    created_at: DateTime<Utc>,
}

async fn list_system_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SystemLogQuery>,
) -> ApiResult<Json<Vec<SystemLogRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let rows = sqlx::query_as::<_, SystemLogRow>(
        "SELECT id, service_name, severity, message, context, created_at
         FROM system_error_logs
         WHERE ($1::text IS NULL OR service_name = $1)
           AND ($2::text IS NULL OR severity = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(query.service_name)
    .bind(query.severity)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_system_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SystemLogCreate>,
) -> ApiResult<Json<SystemLogRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_severity(&payload.severity)?;

    let row = sqlx::query_as::<_, SystemLogRow>(
        "INSERT INTO system_error_logs (service_name, severity, message, context)
         VALUES ($1, $2, $3, $4)
         RETURNING id, service_name, severity, message, context, created_at",
    )
    .bind(payload.service_name)
    .bind(payload.severity)
    .bind(payload.message)
    .bind(payload.context.map(SqlJson))
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "system_error_logs",
        Some(row.id),
        "create",
        None,
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn get_system_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<SystemLogRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let row = sqlx::query_as::<_, SystemLogRow>(
        "SELECT id, service_name, severity, message, context, created_at
         FROM system_error_logs
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(row))
}

fn validate_severity(severity: &str) -> ApiResult<()> {
    match severity {
        "debug" | "info" | "warning" | "error" | "critical" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid severity".to_string())),
    }
}
