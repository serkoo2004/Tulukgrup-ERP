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
        .route("/", get(list_policies).post(create_policy))
        .route(
            "/:id",
            get(get_policy)
                .patch(update_policy_renewal_status)
                .delete(archive_policy),
        )
}

#[derive(Debug, Deserialize)]
struct PolicyQuery {
    vehicle_id: Option<i64>,
    policy_type: Option<String>,
    renewal_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PolicyCreate {
    vehicle_id: i64,
    policy_type: String,
    policy_number: String,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    previous_amount: Option<Decimal>,
    currency: Option<String>,
    pdf_file: Option<String>,
    renewal_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RenewalStatusUpdate {
    renewal_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct PolicyRow {
    id: i64,
    vehicle_id: i64,
    policy_type: String,
    policy_number: String,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    previous_amount: Option<Decimal>,
    currency: String,
    pdf_file: Option<String>,
    renewal_status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow)]
struct VehicleAccessRow {
    status: String,
    is_active: bool,
}

async fn list_policies(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PolicyQuery>,
) -> ApiResult<Json<Vec<PolicyRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, PolicyRow>(
        "SELECT id, vehicle_id, policy_type::text AS policy_type, policy_number,
                insurance_company, agency_name, start_date, end_date, amount,
                previous_amount, currency, pdf_file, renewal_status::text AS renewal_status,
                created_at, updated_at, created_by
         FROM insurance_policies
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::text IS NULL OR policy_type::text = $2)
           AND ($3::text IS NULL OR renewal_status::text = $3)
         ORDER BY end_date ASC NULLS LAST, id DESC
         LIMIT $4",
    )
    .bind(query.vehicle_id)
    .bind(query.policy_type)
    .bind(query.renewal_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PolicyCreate>,
) -> ApiResult<Json<PolicyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_policy_type(&payload.policy_type)?;

    let vehicle = find_vehicle_for_policy(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_allows_policy(&vehicle)?;

    let renewal_status = payload
        .renewal_status
        .unwrap_or_else(|| "active".to_string());
    validate_renewal_status(&renewal_status)?;
    let currency = payload.currency.unwrap_or_else(|| "TRY".to_string());

    let row = sqlx::query_as::<_, PolicyRow>(
        "INSERT INTO insurance_policies
         (vehicle_id, policy_type, policy_number, insurance_company, agency_name,
          start_date, end_date, amount, previous_amount, currency, pdf_file,
          renewal_status, created_by)
         VALUES ($1, $2::policy_type, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                 $12::renewal_status, $13)
         RETURNING id, vehicle_id, policy_type::text AS policy_type, policy_number,
                   insurance_company, agency_name, start_date, end_date, amount,
                   previous_amount, currency, pdf_file, renewal_status::text AS renewal_status,
                   created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.policy_type)
    .bind(payload.policy_number)
    .bind(payload.insurance_company)
    .bind(payload.agency_name)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .bind(payload.amount)
    .bind(payload.previous_amount)
    .bind(currency)
    .bind(payload.pdf_file)
    .bind(renewal_status)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "insurance_policies",
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

async fn get_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<PolicyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_policy(&state.pool, id).await?))
}

async fn update_policy_renewal_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<RenewalStatusUpdate>,
) -> ApiResult<Json<PolicyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_renewal_status(&payload.renewal_status)?;

    let old = find_policy(&state.pool, id).await?;
    let row = sqlx::query_as::<_, PolicyRow>(
        "UPDATE insurance_policies
         SET renewal_status = $2::renewal_status,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, policy_type::text AS policy_type, policy_number,
                   insurance_company, agency_name, start_date, end_date, amount,
                   previous_amount, currency, pdf_file, renewal_status::text AS renewal_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.renewal_status)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "insurance_policies",
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

async fn archive_policy(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<PolicyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_policy(&state.pool, id).await?;

    let row = sqlx::query_as::<_, PolicyRow>(
        "UPDATE insurance_policies
         SET deleted_at = now(),
             is_active = false,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, policy_type::text AS policy_type, policy_number,
                   insurance_company, agency_name, start_date, end_date, amount,
                   previous_amount, currency, pdf_file, renewal_status::text AS renewal_status,
                   created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "insurance_policies",
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

async fn find_policy(pool: &PgPool, id: i64) -> ApiResult<PolicyRow> {
    sqlx::query_as::<_, PolicyRow>(
        "SELECT id, vehicle_id, policy_type::text AS policy_type, policy_number,
                insurance_company, agency_name, start_date, end_date, amount,
                previous_amount, currency, pdf_file, renewal_status::text AS renewal_status,
                created_at, updated_at, created_by
         FROM insurance_policies
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_for_policy(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleAccessRow> {
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

fn ensure_vehicle_allows_policy(vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot receive new policy records".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_policy_type(policy_type: &str) -> ApiResult<()> {
    match policy_type {
        "trafik" | "kasko" | "imm" | "ferdi_kaza" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid policy type".to_string())),
    }
}

fn validate_renewal_status(status: &str) -> ApiResult<()> {
    match status {
        "active" | "approaching" | "renewing" | "ended" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid renewal status".to_string())),
    }
}
