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
        .route("/", get(list_maintenances).post(create_maintenance))
        .route(
            "/:id",
            get(get_maintenance)
                .patch(update_maintenance_status)
                .delete(archive_maintenance),
        )
}

#[derive(Debug, Deserialize)]
struct MaintenanceQuery {
    vehicle_id: Option<i64>,
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct MaintenanceCreate {
    vehicle_id: i64,
    last_maintenance_km: Option<i32>,
    next_maintenance_km: Option<i32>,
    maintenance_date: Option<NaiveDate>,
    service_company: Option<String>,
    maintenance_type: Option<String>,
    description: Option<String>,
    total_cost: Option<Decimal>,
    invoice_file: Option<String>,
    maintenance_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MaintenanceStatusUpdate {
    maintenance_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct MaintenanceRow {
    id: i64,
    vehicle_id: i64,
    last_maintenance_km: Option<i32>,
    next_maintenance_km: Option<i32>,
    maintenance_date: Option<NaiveDate>,
    service_company: Option<String>,
    maintenance_type: Option<String>,
    description: Option<String>,
    total_cost: Option<Decimal>,
    invoice_file: Option<String>,
    maintenance_status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow)]
struct VehicleAccessRow {
    status: String,
    is_active: bool,
}

async fn list_maintenances(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MaintenanceQuery>,
) -> ApiResult<Json<Vec<MaintenanceRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = sqlx::query_as::<_, MaintenanceRow>(
        "SELECT id, vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
                service_company, maintenance_type, description, total_cost, invoice_file,
                maintenance_status::text AS maintenance_status, created_at, updated_at, created_by
         FROM maintenances
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::text IS NULL OR maintenance_status::text = $2)
         ORDER BY COALESCE(maintenance_date, created_at::date) DESC, id DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_maintenance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<MaintenanceCreate>,
) -> ApiResult<Json<MaintenanceRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let vehicle = find_vehicle_for_maintenance(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_is_operational(&vehicle)?;

    let status = payload
        .maintenance_status
        .unwrap_or_else(|| "planned".to_string());
    validate_maintenance_status(&status)?;

    let row = sqlx::query_as::<_, MaintenanceRow>(
        "INSERT INTO maintenances
         (vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
          service_company, maintenance_type, description, total_cost, invoice_file,
          maintenance_status, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::maintenance_status, $11)
         RETURNING id, vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
                   service_company, maintenance_type, description, total_cost, invoice_file,
                   maintenance_status::text AS maintenance_status, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.last_maintenance_km)
    .bind(payload.next_maintenance_km)
    .bind(payload.maintenance_date)
    .bind(payload.service_company)
    .bind(payload.maintenance_type)
    .bind(payload.description)
    .bind(payload.total_cost)
    .bind(payload.invoice_file)
    .bind(status)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "maintenances",
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

async fn get_maintenance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<MaintenanceRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_maintenance(&state.pool, id).await?))
}

async fn update_maintenance_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<MaintenanceStatusUpdate>,
) -> ApiResult<Json<MaintenanceRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_maintenance_status(&payload.maintenance_status)?;

    let old = find_maintenance(&state.pool, id).await?;
    let row = sqlx::query_as::<_, MaintenanceRow>(
        "UPDATE maintenances
         SET maintenance_status = $2::maintenance_status,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
                   service_company, maintenance_type, description, total_cost, invoice_file,
                   maintenance_status::text AS maintenance_status, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.maintenance_status)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "maintenances",
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

async fn archive_maintenance(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<MaintenanceRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_maintenance(&state.pool, id).await?;

    let row = sqlx::query_as::<_, MaintenanceRow>(
        "UPDATE maintenances
         SET deleted_at = now(),
             is_active = false,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
                   service_company, maintenance_type, description, total_cost, invoice_file,
                   maintenance_status::text AS maintenance_status, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "maintenances",
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

async fn find_maintenance(pool: &PgPool, id: i64) -> ApiResult<MaintenanceRow> {
    sqlx::query_as::<_, MaintenanceRow>(
        "SELECT id, vehicle_id, last_maintenance_km, next_maintenance_km, maintenance_date,
                service_company, maintenance_type, description, total_cost, invoice_file,
                maintenance_status::text AS maintenance_status, created_at, updated_at, created_by
         FROM maintenances
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_for_maintenance(
    pool: &PgPool,
    vehicle_id: i64,
) -> ApiResult<VehicleAccessRow> {
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

fn ensure_vehicle_is_operational(vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot receive maintenance records".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_maintenance_status(status: &str) -> ApiResult<()> {
    match status {
        "planned" | "scheduled" | "completed" | "cancelled" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid maintenance status".to_string(),
        )),
    }
}
