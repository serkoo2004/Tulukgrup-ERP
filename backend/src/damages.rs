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
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_damages).post(create_damage))
        .route(
            "/:id",
            get(get_damage).patch(update_damage).delete(archive_damage),
        )
}

#[derive(Debug, Deserialize)]
struct DamageQuery {
    vehicle_id: Option<i64>,
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct DamageCreate {
    vehicle_id: i64,
    damage_date: NaiveDate,
    damage_type: Option<String>,
    description: Option<String>,
    estimated_cost: Option<Decimal>,
    actual_cost: Option<Decimal>,
    insurance_claim_no: Option<String>,
    damage_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DamageUpdate {
    damage_status: Option<String>,
    estimated_cost: Option<Decimal>,
    actual_cost: Option<Decimal>,
    insurance_claim_no: Option<String>,
    description: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct DamageRow {
    id: i64,
    vehicle_id: i64,
    damage_date: NaiveDate,
    damage_type: Option<String>,
    description: Option<String>,
    estimated_cost: Option<Decimal>,
    actual_cost: Option<Decimal>,
    insurance_claim_no: Option<String>,
    damage_status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow)]
struct VehicleAccessRow {
    status: String,
    is_active: bool,
}

async fn list_damages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DamageQuery>,
) -> ApiResult<Json<Vec<DamageRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, DamageRow>(
        "SELECT id, vehicle_id, damage_date, damage_type, description, estimated_cost,
                actual_cost, insurance_claim_no, damage_status::text AS damage_status,
                created_at, updated_at, created_by
         FROM damages
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::text IS NULL OR damage_status::text = $2)
         ORDER BY damage_date DESC, created_at DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_damage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DamageCreate>,
) -> ApiResult<Json<DamageRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let vehicle = find_vehicle_for_damage(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_allows_damage(&vehicle)?;

    let status = payload.damage_status.unwrap_or_else(|| "open".to_string());
    validate_damage_status(&status)?;

    let row = sqlx::query_as::<_, DamageRow>(
        "INSERT INTO damages
         (vehicle_id, damage_date, damage_type, description, estimated_cost, actual_cost,
          insurance_claim_no, damage_status, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8::damage_status, $9)
         RETURNING id, vehicle_id, damage_date, damage_type, description, estimated_cost,
                   actual_cost, insurance_claim_no, damage_status::text AS damage_status,
                   created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.damage_date)
    .bind(payload.damage_type)
    .bind(payload.description)
    .bind(payload.estimated_cost)
    .bind(payload.actual_cost)
    .bind(payload.insurance_claim_no)
    .bind(status)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "damages",
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

async fn get_damage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<DamageRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_damage(&state.pool, id).await?))
}

async fn update_damage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<DamageUpdate>,
) -> ApiResult<Json<DamageRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    if let Some(status) = payload.damage_status.as_deref() {
        validate_damage_status(status)?;
    }

    let old = find_damage(&state.pool, id).await?;
    let row = sqlx::query_as::<_, DamageRow>(
        "UPDATE damages SET
            damage_status = COALESCE($2::damage_status, damage_status),
            estimated_cost = COALESCE($3, estimated_cost),
            actual_cost = COALESCE($4, actual_cost),
            insurance_claim_no = COALESCE($5, insurance_claim_no),
            description = COALESCE($6, description),
            updated_by = $7,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, damage_date, damage_type, description, estimated_cost,
                   actual_cost, insurance_claim_no, damage_status::text AS damage_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.damage_status)
    .bind(payload.estimated_cost)
    .bind(payload.actual_cost)
    .bind(payload.insurance_claim_no)
    .bind(payload.description)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "damages",
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

async fn archive_damage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<DamageRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_damage(&state.pool, id).await?;

    let row = sqlx::query_as::<_, DamageRow>(
        "UPDATE damages
         SET deleted_at = now(),
             is_active = false,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, damage_date, damage_type, description, estimated_cost,
                   actual_cost, insurance_claim_no, damage_status::text AS damage_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "damages",
        Some(row.id),
        "delete",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn find_damage(pool: &PgPool, id: i64) -> ApiResult<DamageRow> {
    sqlx::query_as::<_, DamageRow>(
        "SELECT id, vehicle_id, damage_date, damage_type, description, estimated_cost,
                actual_cost, insurance_claim_no, damage_status::text AS damage_status,
                created_at, updated_at, created_by
         FROM damages
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_for_damage(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleAccessRow> {
    sqlx::query_as::<_, VehicleAccessRow>(
        "SELECT status::text AS status, is_active
         FROM vehicles
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn ensure_vehicle_allows_damage(vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot receive damage records".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_damage_status(status: &str) -> ApiResult<()> {
    match status {
        "open" | "expertise" | "insurance" | "repaired" | "closed" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid damage status".to_string())),
    }
}
