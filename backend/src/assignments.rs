use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_assignments).post(assign_vehicle))
        .route("/:id/release", post(release_assignment))
        .route("/vehicles/:vehicle_id", get(list_vehicle_assignments))
}

#[derive(Debug, Deserialize)]
struct AssignmentQuery {
    vehicle_id: Option<i64>,
    user_id: Option<i64>,
    active_only: Option<bool>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct AssignVehicleRequest {
    vehicle_id: i64,
    user_id: i64,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAssignmentRequest {
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct AssignmentRow {
    id: i64,
    vehicle_id: i64,
    user_id: i64,
    assigned_at: DateTime<Utc>,
    released_at: Option<DateTime<Utc>>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow)]
struct VehicleStatusRow {
    status: String,
    is_active: bool,
}

async fn list_assignments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AssignmentQuery>,
) -> ApiResult<Json<Vec<AssignmentRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(query_assignments(&state.pool, query).await?))
}

async fn list_vehicle_assignments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vehicle_id): Path<i64>,
    Query(mut query): Query<AssignmentQuery>,
) -> ApiResult<Json<Vec<AssignmentRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    query.vehicle_id = Some(vehicle_id);
    Ok(Json(query_assignments(&state.pool, query).await?))
}

async fn assign_vehicle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AssignVehicleRequest>,
) -> ApiResult<Json<AssignmentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let vehicle = find_vehicle_status(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_assignable(&vehicle)?;

    let mut tx = state.pool.begin().await?;
    close_open_assignments(
        &mut tx,
        payload.vehicle_id,
        current.id,
        payload.note.clone(),
    )
    .await?;

    let row = sqlx::query_as::<_, AssignmentRow>(
        "INSERT INTO vehicle_assignments
         (vehicle_id, user_id, assigned_at, note, created_by)
         VALUES ($1, $2, now(), $3, $4)
         RETURNING id, vehicle_id, user_id, assigned_at, released_at, note,
                   created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.user_id)
    .bind(payload.note)
    .bind(current.id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE vehicles
         SET user_id = $2,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(payload.vehicle_id)
    .bind(payload.user_id)
    .bind(current.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::write_audit(
        &state.pool,
        "vehicle_assignments",
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

async fn release_assignment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ReleaseAssignmentRequest>,
) -> ApiResult<Json<AssignmentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let old = find_assignment(&state.pool, id).await?;

    let mut tx = state.pool.begin().await?;
    let row = sqlx::query_as::<_, AssignmentRow>(
        "UPDATE vehicle_assignments
         SET released_at = COALESCE(released_at, now()),
             note = COALESCE($2, note),
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, user_id, assigned_at, released_at, note,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.note)
    .bind(current.id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;

    sqlx::query(
        "UPDATE vehicles
         SET user_id = NULL,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND user_id = $3",
    )
    .bind(row.vehicle_id)
    .bind(current.id)
    .bind(row.user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::write_audit(
        &state.pool,
        "vehicle_assignments",
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

async fn query_assignments(pool: &PgPool, query: AssignmentQuery) -> ApiResult<Vec<AssignmentRow>> {
    Ok(sqlx::query_as::<_, AssignmentRow>(
        "SELECT id, vehicle_id, user_id, assigned_at, released_at, note,
                created_at, updated_at, created_by
         FROM vehicle_assignments
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::bigint IS NULL OR user_id = $2)
           AND ($3::boolean = false OR released_at IS NULL)
         ORDER BY assigned_at DESC
         LIMIT $4",
    )
    .bind(query.vehicle_id)
    .bind(query.user_id)
    .bind(query.active_only.unwrap_or(false))
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(pool)
    .await?)
}

async fn find_assignment(pool: &PgPool, id: i64) -> ApiResult<AssignmentRow> {
    sqlx::query_as::<_, AssignmentRow>(
        "SELECT id, vehicle_id, user_id, assigned_at, released_at, note,
                created_at, updated_at, created_by
         FROM vehicle_assignments
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_status(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleStatusRow> {
    sqlx::query_as::<_, VehicleStatusRow>(
        "SELECT status::text AS status, is_active
         FROM vehicles
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn close_open_assignments(
    tx: &mut Transaction<'_, Postgres>,
    vehicle_id: i64,
    actor_id: i64,
    note: Option<String>,
) -> ApiResult<()> {
    sqlx::query(
        "UPDATE vehicle_assignments
         SET released_at = now(),
             note = COALESCE($3, note),
             updated_by = $2,
             updated_at = now()
         WHERE vehicle_id = $1 AND released_at IS NULL AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .bind(actor_id)
    .bind(note)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn ensure_vehicle_assignable(vehicle: &VehicleStatusRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot be assigned".to_string(),
        ))
    } else {
        Ok(())
    }
}
