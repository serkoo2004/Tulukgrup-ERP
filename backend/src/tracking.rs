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
use serde_json::{json, Value};
use sqlx::{types::Json as SqlJson, FromRow, PgPool};

const DEFAULT_ABNORMAL_KM_INCREASE: i32 = 5_000;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/km-logs", get(list_km_logs).post(create_km_log))
        .route("/km-logs/:id/verify", post(verify_km_log))
        .route("/vehicles/:vehicle_id/km-logs", get(list_vehicle_km_logs))
}

#[derive(Debug, Deserialize)]
struct KmLogQuery {
    vehicle_id: Option<i64>,
    verification_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct KmLogCreate {
    vehicle_id: i64,
    km: i32,
    entry_type: Option<String>,
    image_path: Option<String>,
    ocr_result: Option<Value>,
    device_info: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VerifyKmLogRequest {
    verification_status: String,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct KmLogRow {
    id: i64,
    vehicle_id: i64,
    km: i32,
    entry_type: String,
    image_path: Option<String>,
    ocr_result: Option<Value>,
    verification_status: String,
    ip_address: Option<String>,
    device_info: Option<String>,
    created_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow)]
struct VehicleAccessRow {
    id: i64,
    plate: String,
    status: String,
    is_active: bool,
    user_id: Option<i64>,
}

async fn list_km_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<KmLogQuery>,
) -> ApiResult<Json<Vec<KmLogRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = sqlx::query_as::<_, KmLogRow>(
        "SELECT id, vehicle_id, km, entry_type::text AS entry_type, image_path, ocr_result,
                verification_status::text AS verification_status, ip_address::text AS ip_address,
                device_info, created_at, created_by
         FROM km_logs
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::text IS NULL OR verification_status::text = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.verification_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn list_vehicle_km_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(vehicle_id): Path<i64>,
    Query(query): Query<KmLogQuery>,
) -> ApiResult<Json<Vec<KmLogRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    let vehicle = find_vehicle_for_access(&state.pool, vehicle_id).await?;
    ensure_vehicle_read_access(&current, &vehicle)?;

    let rows = sqlx::query_as::<_, KmLogRow>(
        "SELECT id, vehicle_id, km, entry_type::text AS entry_type, image_path, ocr_result,
                verification_status::text AS verification_status, ip_address::text AS ip_address,
                device_info, created_at, created_by
         FROM km_logs
         WHERE deleted_at IS NULL AND vehicle_id = $1
           AND ($2::text IS NULL OR verification_status::text = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(vehicle_id)
    .bind(query.verification_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_km_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<KmLogCreate>,
) -> ApiResult<Json<KmLogRow>> {
    let current = auth::current_user(&state, &headers).await?;
    let vehicle = find_vehicle_for_access(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_write_access(&state.pool, &current, &vehicle).await?;
    ensure_vehicle_accepts_km(&vehicle)?;

    if payload.km < 0 {
        return Err(ApiError::BadRequest("km cannot be negative".to_string()));
    }

    let entry_type = payload.entry_type.unwrap_or_else(|| "manual".to_string());
    validate_entry_type(&entry_type)?;
    if entry_type == "ocr" && payload.image_path.as_deref().unwrap_or_default().trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "image_path is required for ocr km logs".to_string(),
        ));
    }

    let previous_km = latest_km_for_vehicle(&state.pool, payload.vehicle_id).await?;
    let verification_status = classify_km(payload.km, previous_km);
    let ip_address = header_ip(&headers);

    let row = sqlx::query_as::<_, KmLogRow>(
        "INSERT INTO km_logs
         (vehicle_id, km, entry_type, image_path, ocr_result, verification_status,
          ip_address, device_info, created_by)
         VALUES ($1, $2, $3::km_entry_type, $4, $5, $6::verification_status,
                 $7::inet, $8, $9)
         RETURNING id, vehicle_id, km, entry_type::text AS entry_type, image_path, ocr_result,
                   verification_status::text AS verification_status, ip_address::text AS ip_address,
                   device_info, created_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.km)
    .bind(entry_type)
    .bind(payload.image_path)
    .bind(payload.ocr_result.map(SqlJson))
    .bind(verification_status)
    .bind(ip_address)
    .bind(payload.device_info)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    if row.verification_status == "suspicious" {
        create_km_review_task(&state.pool, &vehicle, &row, current.id).await?;
    }
    if row.entry_type == "ocr" {
        create_km_ocr_ai_job(&state.pool, &vehicle, &row, current.id).await?;
    }

    audit::write_audit(
        &state.pool,
        "km_logs",
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

async fn verify_km_log(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<VerifyKmLogRequest>,
) -> ApiResult<Json<KmLogRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_verification_status(&payload.verification_status)?;

    let old = find_km_log(&state.pool, id).await?;
    let row = sqlx::query_as::<_, KmLogRow>(
        "UPDATE km_logs
         SET verification_status = $2::verification_status,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, km, entry_type::text AS entry_type, image_path, ocr_result,
                   verification_status::text AS verification_status, ip_address::text AS ip_address,
                   device_info, created_at, created_by",
    )
    .bind(id)
    .bind(payload.verification_status)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "km_logs",
        Some(row.id),
        "update",
        Some(serde_json::to_value(&old).unwrap_or_else(|_| json!({}))),
        json!({
            "km_log": serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
            "verification_note": payload.note
        }),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn find_vehicle_for_access(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleAccessRow> {
    sqlx::query_as::<_, VehicleAccessRow>(
        "SELECT id, plate, status::text AS status, is_active, user_id
         FROM vehicles
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_km_log(pool: &PgPool, id: i64) -> ApiResult<KmLogRow> {
    sqlx::query_as::<_, KmLogRow>(
        "SELECT id, vehicle_id, km, entry_type::text AS entry_type, image_path, ocr_result,
                verification_status::text AS verification_status, ip_address::text AS ip_address,
                device_info, created_at, created_by
         FROM km_logs
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn latest_km_for_vehicle(pool: &PgPool, vehicle_id: i64) -> ApiResult<Option<i32>> {
    Ok(sqlx::query_scalar::<_, i32>(
        "SELECT km
         FROM km_logs
         WHERE vehicle_id = $1 AND deleted_at IS NULL AND verification_status != 'rejected'::verification_status
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(vehicle_id)
    .fetch_optional(pool)
    .await?)
}

async fn create_km_review_task(
    pool: &PgPool,
    vehicle: &VehicleAccessRow,
    row: &KmLogRow,
    actor_id: i64,
) -> ApiResult<()> {
    sqlx::query(
        "INSERT INTO tasks
         (task_type, related_vehicle_id, priority, task_status, description, created_by)
         VALUES ('km_review', $1, 'high', 'open', $2, $3)",
    )
    .bind(vehicle.id)
    .bind(format!(
        "{} plakali arac icin supheli KM girisi: {} km",
        vehicle.plate, row.km
    ))
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_km_ocr_ai_job(
    pool: &PgPool,
    vehicle: &VehicleAccessRow,
    row: &KmLogRow,
    actor_id: i64,
) -> ApiResult<()> {
    let input_data = json!({
        "vehicle": {
            "id": vehicle.id,
            "plate": vehicle.plate,
            "status": vehicle.status
        },
        "km_log": {
            "id": row.id,
            "vehicle_id": row.vehicle_id,
            "submitted_km": row.km,
            "entry_type": row.entry_type,
            "image_path": row.image_path,
            "verification_status": row.verification_status,
            "created_at": row.created_at
        },
        "ocr_contract": {
            "task": "read_odometer_from_image",
            "expected_output": ["detected_km", "confidence", "evidence", "mismatch_with_submitted_km"],
            "do_not_auto_verify": true,
            "human_review_required": true
        }
    });

    sqlx::query(
        "INSERT INTO ai_analysis_jobs
         (analysis_type, related_vehicle_id, source_table, source_record_id,
          input_data, requires_human_approval, created_by)
         VALUES ('ocr_verification', $1, 'km_logs', $2, $3, true, $4)",
    )
    .bind(row.vehicle_id)
    .bind(row.id)
    .bind(SqlJson(input_data))
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn ensure_vehicle_read_access(user: &auth::AuthUser, vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if matches!(
        user.role.as_str(),
        "admin" | "manager" | "operation" | "accounting"
    ) || vehicle.user_id == Some(user.id)
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

async fn ensure_vehicle_write_access(
    pool: &PgPool,
    user: &auth::AuthUser,
    vehicle: &VehicleAccessRow,
) -> ApiResult<()> {
    if matches!(user.role.as_str(), "admin" | "manager" | "operation")
        || vehicle.user_id == Some(user.id)
        || auth::has_mobile_permission(pool, user, "km_log", "create").await?
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

fn ensure_vehicle_accepts_km(vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot receive km logs".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn classify_km(new_km: i32, previous_km: Option<i32>) -> &'static str {
    match previous_km {
        Some(previous) if new_km < previous => "suspicious",
        Some(previous) if new_km - previous > DEFAULT_ABNORMAL_KM_INCREASE => "suspicious",
        _ => "pending",
    }
}

fn validate_entry_type(entry_type: &str) -> ApiResult<()> {
    match entry_type {
        "manual" | "ocr" | "mobiliz_api" | "kopilot_api" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid km entry type".to_string())),
    }
}

fn validate_verification_status(status: &str) -> ApiResult<()> {
    match status {
        "pending" | "verified" | "suspicious" | "rejected" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid verification status".to_string(),
        )),
    }
}

fn header_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
