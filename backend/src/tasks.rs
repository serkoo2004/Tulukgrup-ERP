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
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tasks).post(create_task))
        .route("/mine", get(list_my_tasks))
        .route("/:id", get(get_task).patch(update_task_status))
}

#[derive(Debug, Deserialize)]
struct TaskQuery {
    task_type: Option<String>,
    related_vehicle_id: Option<i64>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    task_status: Option<String>,
    priority: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct TaskCreate {
    task_type: String,
    related_vehicle_id: Option<i64>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    priority: Option<String>,
    due_date: Option<NaiveDate>,
    task_status: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TaskStatusUpdate {
    task_status: Option<String>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    priority: Option<String>,
    due_date: Option<NaiveDate>,
    description: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct TaskRow {
    id: i64,
    task_type: String,
    related_vehicle_id: Option<i64>,
    assigned_user_id: Option<i64>,
    assigned_department_id: Option<i64>,
    priority: String,
    due_date: Option<NaiveDate>,
    task_status: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

async fn list_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TaskQuery>,
) -> ApiResult<Json<Vec<TaskRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = sqlx::query_as::<_, TaskRow>(
        "SELECT id, task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
                priority::text AS priority, due_date, task_status, description,
                created_at, updated_at, created_by
         FROM tasks
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR task_type = $1)
           AND ($2::bigint IS NULL OR related_vehicle_id = $2)
           AND ($3::bigint IS NULL OR assigned_user_id = $3)
           AND ($4::bigint IS NULL OR assigned_department_id = $4)
           AND ($5::text IS NULL OR task_status = $5)
           AND ($6::text IS NULL OR priority::text = $6)
         ORDER BY
           CASE priority::text
             WHEN 'critical' THEN 1
             WHEN 'high' THEN 2
             WHEN 'medium' THEN 3
             ELSE 4
           END,
           due_date ASC NULLS LAST,
           created_at DESC
         LIMIT $7",
    )
    .bind(query.task_type)
    .bind(query.related_vehicle_id)
    .bind(query.assigned_user_id)
    .bind(query.assigned_department_id)
    .bind(query.task_status)
    .bind(query.priority)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn list_my_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<TaskQuery>,
) -> ApiResult<Json<Vec<TaskRow>>> {
    let current = auth::current_user(&state, &headers).await?;

    let rows = sqlx::query_as::<_, TaskRow>(
        "SELECT id, task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
                priority::text AS priority, due_date, task_status, description,
                created_at, updated_at, created_by
         FROM tasks
         WHERE deleted_at IS NULL
           AND assigned_user_id = $1
           AND ($2::text IS NULL OR task_status = $2)
           AND ($3::text IS NULL OR priority::text = $3)
         ORDER BY
           CASE priority::text
             WHEN 'critical' THEN 1
             WHEN 'high' THEN 2
             WHEN 'medium' THEN 3
             ELSE 4
           END,
           due_date ASC NULLS LAST,
           created_at DESC
         LIMIT $4",
    )
    .bind(current.id)
    .bind(query.task_status)
    .bind(query.priority)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TaskCreate>,
) -> ApiResult<Json<TaskRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let priority = payload.priority.unwrap_or_else(|| "medium".to_string());
    validate_priority(&priority)?;
    let status = payload.task_status.unwrap_or_else(|| "open".to_string());
    validate_task_status(&status)?;

    let row = sqlx::query_as::<_, TaskRow>(
        "INSERT INTO tasks
         (task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
          priority, due_date, task_status, description, created_by)
         VALUES ($1, $2, $3, $4, $5::priority_level, $6, $7, $8, $9)
         RETURNING id, task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
                   priority::text AS priority, due_date, task_status, description,
                   created_at, updated_at, created_by",
    )
    .bind(payload.task_type)
    .bind(payload.related_vehicle_id)
    .bind(payload.assigned_user_id)
    .bind(payload.assigned_department_id)
    .bind(priority)
    .bind(payload.due_date)
    .bind(status)
    .bind(payload.description)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "tasks",
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

async fn get_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<TaskRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let row = find_task(&state.pool, id).await?;
    if row.assigned_user_id == Some(current.id)
        || matches!(current.role.as_str(), "admin" | "manager" | "operation")
    {
        Ok(Json(row))
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn update_task_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<TaskStatusUpdate>,
) -> ApiResult<Json<TaskRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let old = find_task(&state.pool, id).await?;
    if old.assigned_user_id != Some(current.id) {
        auth::require_roles(&current, &["admin", "manager", "operation"])?;
    }
    if let Some(status) = payload.task_status.as_deref() {
        validate_task_status(status)?;
    }
    if let Some(priority) = payload.priority.as_deref() {
        validate_priority(priority)?;
    }

    let row = sqlx::query_as::<_, TaskRow>(
        "UPDATE tasks
         SET task_status = COALESCE($2, task_status),
             assigned_user_id = COALESCE($3, assigned_user_id),
             assigned_department_id = COALESCE($4, assigned_department_id),
             priority = COALESCE($5::priority_level, priority),
             due_date = COALESCE($6, due_date),
             description = COALESCE($7, description),
             updated_by = $8,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
                   priority::text AS priority, due_date, task_status, description,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.task_status)
    .bind(payload.assigned_user_id)
    .bind(payload.assigned_department_id)
    .bind(payload.priority)
    .bind(payload.due_date)
    .bind(payload.description)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "tasks",
        Some(row.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn find_task(pool: &sqlx::PgPool, id: i64) -> ApiResult<TaskRow> {
    sqlx::query_as::<_, TaskRow>(
        "SELECT id, task_type, related_vehicle_id, assigned_user_id, assigned_department_id,
                priority::text AS priority, due_date, task_status, description,
                created_at, updated_at, created_by
         FROM tasks
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn validate_priority(priority: &str) -> ApiResult<()> {
    match priority {
        "low" | "medium" | "high" | "critical" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid priority".to_string())),
    }
}

fn validate_task_status(status: &str) -> ApiResult<()> {
    match status {
        "open" | "in_progress" | "waiting" | "completed" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid task status".to_string())),
    }
}
