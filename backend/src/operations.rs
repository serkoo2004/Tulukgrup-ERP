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
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/vehicle-inspections",
            get(list_vehicle_inspections).post(create_vehicle_inspection),
        )
        .route(
            "/vehicle-inspections/:id",
            get(get_vehicle_inspection)
                .patch(update_vehicle_inspection)
                .delete(archive_vehicle_inspection),
        )
        .route(
            "/value-loss-claims",
            get(list_value_loss_claims).post(create_value_loss_claim),
        )
        .route(
            "/value-loss-claims/:id",
            get(get_value_loss_claim)
                .patch(update_value_loss_claim)
                .delete(archive_value_loss_claim),
        )
        .route(
            "/fuel-entries",
            get(list_fuel_entries).post(create_fuel_entry),
        )
        .route(
            "/fuel-entries/:id",
            get(get_fuel_entry)
                .patch(update_fuel_entry)
                .delete(archive_fuel_entry),
        )
        .route(
            "/vehicle-washes",
            get(list_vehicle_washes).post(create_vehicle_wash),
        )
        .route(
            "/vehicle-washes/:id",
            get(get_vehicle_wash)
                .patch(update_vehicle_wash)
                .delete(archive_vehicle_wash),
        )
        .route(
            "/insurance-quotes",
            get(list_insurance_quotes).post(create_insurance_quote),
        )
        .route(
            "/insurance-quotes/:id",
            get(get_insurance_quote)
                .patch(update_insurance_quote)
                .delete(archive_insurance_quote),
        )
}

#[derive(Debug, Deserialize)]
struct ListQuery {
    vehicle_id: Option<i64>,
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct VehicleInspectionCreate {
    vehicle_id: i64,
    branch: Option<String>,
    inspection_date: NaiveDate,
    inspector_user_id: Option<i64>,
    exterior_ok: Option<bool>,
    interior_ok: Option<bool>,
    equipment_ok: Option<bool>,
    documents_ok: Option<bool>,
    damage_note: Option<String>,
    action_note: Option<String>,
    expert_report: Option<String>,
    fee: Option<Decimal>,
    inspection_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VehicleInspectionUpdate {
    branch: Option<String>,
    inspection_date: Option<NaiveDate>,
    inspector_user_id: Option<i64>,
    exterior_ok: Option<bool>,
    interior_ok: Option<bool>,
    equipment_ok: Option<bool>,
    documents_ok: Option<bool>,
    damage_note: Option<String>,
    action_note: Option<String>,
    expert_report: Option<String>,
    fee: Option<Decimal>,
    inspection_status: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleInspectionRow {
    id: i64,
    vehicle_id: i64,
    plate: String,
    branch: Option<String>,
    inspection_date: NaiveDate,
    inspector_user_id: Option<i64>,
    inspector_name: Option<String>,
    exterior_ok: Option<bool>,
    interior_ok: Option<bool>,
    equipment_ok: Option<bool>,
    documents_ok: Option<bool>,
    damage_note: Option<String>,
    action_note: Option<String>,
    expert_report: Option<String>,
    fee: Option<Decimal>,
    inspection_status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ValueLossClaimCreate {
    vehicle_id: i64,
    accident_date: NaiveDate,
    vehicle_purchase_date: Option<NaiveDate>,
    tramer_amount: Option<Decimal>,
    deprivation_days: Option<i32>,
    requested_amount: Option<Decimal>,
    received_amount: Option<Decimal>,
    claim_status: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValueLossClaimUpdate {
    accident_date: Option<NaiveDate>,
    vehicle_purchase_date: Option<NaiveDate>,
    tramer_amount: Option<Decimal>,
    deprivation_days: Option<i32>,
    requested_amount: Option<Decimal>,
    received_amount: Option<Decimal>,
    claim_status: Option<String>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct ValueLossClaimRow {
    id: i64,
    vehicle_id: i64,
    plate: String,
    accident_date: NaiveDate,
    vehicle_purchase_date: Option<NaiveDate>,
    tramer_amount: Option<Decimal>,
    deprivation_days: Option<i32>,
    requested_amount: Option<Decimal>,
    received_amount: Option<Decimal>,
    claim_status: String,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct FuelEntryCreate {
    vehicle_id: i64,
    period_year: i32,
    period_month: i32,
    fuel_limit: Option<Decimal>,
    paid_amount: Option<Decimal>,
    remaining_limit: Option<Decimal>,
    distance_km: Option<i32>,
    current_km: Option<i32>,
    liter_amount: Option<Decimal>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FuelEntryUpdate {
    fuel_limit: Option<Decimal>,
    paid_amount: Option<Decimal>,
    remaining_limit: Option<Decimal>,
    distance_km: Option<i32>,
    current_km: Option<i32>,
    liter_amount: Option<Decimal>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct FuelEntryRow {
    id: i64,
    vehicle_id: i64,
    plate: String,
    assigned_user_name: Option<String>,
    period_year: i32,
    period_month: i32,
    fuel_limit: Option<Decimal>,
    paid_amount: Option<Decimal>,
    remaining_limit: Option<Decimal>,
    distance_km: Option<i32>,
    current_km: Option<i32>,
    liter_amount: Option<Decimal>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct VehicleWashCreate {
    vehicle_id: i64,
    branch: Option<String>,
    wash_company: Option<String>,
    wash_date: NaiveDate,
    amount: Option<Decimal>,
    user_id: Option<i64>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VehicleWashUpdate {
    branch: Option<String>,
    wash_company: Option<String>,
    wash_date: Option<NaiveDate>,
    amount: Option<Decimal>,
    user_id: Option<i64>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct VehicleWashRow {
    id: i64,
    vehicle_id: i64,
    plate: String,
    branch: Option<String>,
    wash_company: Option<String>,
    wash_date: NaiveDate,
    amount: Option<Decimal>,
    user_id: Option<i64>,
    user_name: Option<String>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct InsuranceQuoteCreate {
    vehicle_id: i64,
    quote_type: Option<String>,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    gross_premium: Option<Decimal>,
    installment_count: Option<i32>,
    quote_status: Option<String>,
    valid_until: Option<NaiveDate>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InsuranceQuoteUpdate {
    quote_type: Option<String>,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    gross_premium: Option<Decimal>,
    installment_count: Option<i32>,
    quote_status: Option<String>,
    valid_until: Option<NaiveDate>,
    note: Option<String>,
}

#[derive(Debug, FromRow, Serialize)]
struct InsuranceQuoteRow {
    id: i64,
    vehicle_id: i64,
    plate: String,
    quote_type: String,
    insurance_company: Option<String>,
    agency_name: Option<String>,
    gross_premium: Option<Decimal>,
    installment_count: Option<i32>,
    quote_status: String,
    valid_until: Option<NaiveDate>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

async fn list_vehicle_inspections(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<VehicleInspectionRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, VehicleInspectionRow>(
        "SELECT i.id, i.vehicle_id, v.plate, i.branch, i.inspection_date, i.inspector_user_id,
                u.full_name AS inspector_name, i.exterior_ok, i.interior_ok, i.equipment_ok, i.documents_ok,
                i.damage_note, i.action_note, i.expert_report, i.fee, i.inspection_status,
                i.created_at, i.updated_at, i.created_by
         FROM vehicle_inspections i
         JOIN vehicles v ON v.id = i.vehicle_id
         LEFT JOIN users u ON u.id = i.inspector_user_id
         WHERE i.deleted_at IS NULL
           AND ($1::bigint IS NULL OR i.vehicle_id = $1)
           AND ($2::text IS NULL OR i.inspection_status = $2)
         ORDER BY i.inspection_date DESC, i.created_at DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(limit(query.limit))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_vehicle_inspection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VehicleInspectionCreate>,
) -> ApiResult<Json<VehicleInspectionRow>> {
    let user = authorize_write(&state, &headers).await?;
    let row = sqlx::query_as::<_, VehicleInspectionRow>(
        "INSERT INTO vehicle_inspections
         (vehicle_id, branch, inspection_date, inspector_user_id, exterior_ok, interior_ok, equipment_ok,
          documents_ok, damage_note, action_note, expert_report, fee, inspection_status, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, COALESCE($13, 'open'), $14)
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, inspection_date, inspector_user_id,
                   (SELECT full_name FROM users WHERE id = inspector_user_id) AS inspector_name,
                   exterior_ok, interior_ok, equipment_ok, documents_ok, damage_note, action_note,
                   expert_report, fee, inspection_status, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.branch)
    .bind(payload.inspection_date)
    .bind(payload.inspector_user_id)
    .bind(payload.exterior_ok)
    .bind(payload.interior_ok)
    .bind(payload.equipment_ok)
    .bind(payload.documents_ok)
    .bind(payload.damage_note)
    .bind(payload.action_note)
    .bind(payload.expert_report)
    .bind(payload.fee)
    .bind(payload.inspection_status)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(
        &state,
        &headers,
        "vehicle_inspections",
        row.id,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn get_vehicle_inspection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleInspectionRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_vehicle_inspection(&state, id).await?))
}

async fn update_vehicle_inspection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<VehicleInspectionUpdate>,
) -> ApiResult<Json<VehicleInspectionRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_vehicle_inspection(&state, id).await?;
    let row = sqlx::query_as::<_, VehicleInspectionRow>(
        "UPDATE vehicle_inspections SET
            branch = COALESCE($2, branch),
            inspection_date = COALESCE($3, inspection_date),
            inspector_user_id = COALESCE($4, inspector_user_id),
            exterior_ok = COALESCE($5, exterior_ok),
            interior_ok = COALESCE($6, interior_ok),
            equipment_ok = COALESCE($7, equipment_ok),
            documents_ok = COALESCE($8, documents_ok),
            damage_note = COALESCE($9, damage_note),
            action_note = COALESCE($10, action_note),
            expert_report = COALESCE($11, expert_report),
            fee = COALESCE($12, fee),
            inspection_status = COALESCE($13, inspection_status),
            updated_by = $14,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, inspection_date, inspector_user_id,
                   (SELECT full_name FROM users WHERE id = inspector_user_id) AS inspector_name,
                   exterior_ok, interior_ok, equipment_ok, documents_ok, damage_note, action_note,
                   expert_report, fee, inspection_status, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.branch)
    .bind(payload.inspection_date)
    .bind(payload.inspector_user_id)
    .bind(payload.exterior_ok)
    .bind(payload.interior_ok)
    .bind(payload.equipment_ok)
    .bind(payload.documents_ok)
    .bind(payload.damage_note)
    .bind(payload.action_note)
    .bind(payload.expert_report)
    .bind(payload.fee)
    .bind(payload.inspection_status)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(
        &state,
        &headers,
        "vehicle_inspections",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn archive_vehicle_inspection(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleInspectionRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_vehicle_inspection(&state, id).await?;
    let row = sqlx::query_as::<_, VehicleInspectionRow>(
        "UPDATE vehicle_inspections SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, inspection_date, inspector_user_id,
                   (SELECT full_name FROM users WHERE id = inspector_user_id) AS inspector_name,
                   exterior_ok, interior_ok, equipment_ok, documents_ok, damage_note, action_note,
                   expert_report, fee, inspection_status, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(
        &state,
        &headers,
        "vehicle_inspections",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_value_loss_claims(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<ValueLossClaimRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, ValueLossClaimRow>(
        "SELECT c.id, c.vehicle_id, v.plate, c.accident_date, c.vehicle_purchase_date, c.tramer_amount,
                c.deprivation_days, c.requested_amount, c.received_amount, c.claim_status, c.note,
                c.created_at, c.updated_at, c.created_by
         FROM value_loss_claims c
         JOIN vehicles v ON v.id = c.vehicle_id
         WHERE c.deleted_at IS NULL
           AND ($1::bigint IS NULL OR c.vehicle_id = $1)
           AND ($2::text IS NULL OR c.claim_status = $2)
         ORDER BY c.accident_date DESC, c.created_at DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(limit(query.limit))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_value_loss_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ValueLossClaimCreate>,
) -> ApiResult<Json<ValueLossClaimRow>> {
    let user = authorize_write(&state, &headers).await?;
    let row = sqlx::query_as::<_, ValueLossClaimRow>(
        "INSERT INTO value_loss_claims
         (vehicle_id, accident_date, vehicle_purchase_date, tramer_amount, deprivation_days,
          requested_amount, received_amount, claim_status, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'open'), $9, $10)
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   accident_date, vehicle_purchase_date, tramer_amount, deprivation_days,
                   requested_amount, received_amount, claim_status, note, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.accident_date)
    .bind(payload.vehicle_purchase_date)
    .bind(payload.tramer_amount)
    .bind(payload.deprivation_days)
    .bind(payload.requested_amount)
    .bind(payload.received_amount)
    .bind(payload.claim_status)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(&state, &headers, "value_loss_claims", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn get_value_loss_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ValueLossClaimRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_value_loss_claim(&state, id).await?))
}

async fn update_value_loss_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ValueLossClaimUpdate>,
) -> ApiResult<Json<ValueLossClaimRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_value_loss_claim(&state, id).await?;
    let row = sqlx::query_as::<_, ValueLossClaimRow>(
        "UPDATE value_loss_claims SET
            accident_date = COALESCE($2, accident_date),
            vehicle_purchase_date = COALESCE($3, vehicle_purchase_date),
            tramer_amount = COALESCE($4, tramer_amount),
            deprivation_days = COALESCE($5, deprivation_days),
            requested_amount = COALESCE($6, requested_amount),
            received_amount = COALESCE($7, received_amount),
            claim_status = COALESCE($8, claim_status),
            note = COALESCE($9, note),
            updated_by = $10,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   accident_date, vehicle_purchase_date, tramer_amount, deprivation_days,
                   requested_amount, received_amount, claim_status, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.accident_date)
    .bind(payload.vehicle_purchase_date)
    .bind(payload.tramer_amount)
    .bind(payload.deprivation_days)
    .bind(payload.requested_amount)
    .bind(payload.received_amount)
    .bind(payload.claim_status)
    .bind(payload.note)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(
        &state,
        &headers,
        "value_loss_claims",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn archive_value_loss_claim(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ValueLossClaimRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_value_loss_claim(&state, id).await?;
    let row = sqlx::query_as::<_, ValueLossClaimRow>(
        "UPDATE value_loss_claims SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   accident_date, vehicle_purchase_date, tramer_amount, deprivation_days,
                   requested_amount, received_amount, claim_status, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(
        &state,
        &headers,
        "value_loss_claims",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn list_fuel_entries(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<FuelEntryRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, FuelEntryRow>(
        "SELECT f.id, f.vehicle_id, v.plate, u.full_name AS assigned_user_name, f.period_year, f.period_month,
                f.fuel_limit, f.paid_amount, f.remaining_limit, f.distance_km, f.current_km,
                f.liter_amount, f.note, f.created_at, f.updated_at, f.created_by
         FROM fuel_entries f
         JOIN vehicles v ON v.id = f.vehicle_id
         LEFT JOIN users u ON u.id = v.user_id
         WHERE f.deleted_at IS NULL
           AND ($1::bigint IS NULL OR f.vehicle_id = $1)
         ORDER BY f.period_year DESC, f.period_month DESC, v.plate
         LIMIT $2",
    )
    .bind(query.vehicle_id)
    .bind(limit(query.limit))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_fuel_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FuelEntryCreate>,
) -> ApiResult<Json<FuelEntryRow>> {
    let user = authorize_write(&state, &headers).await?;
    let row = sqlx::query_as::<_, FuelEntryRow>(
        "INSERT INTO fuel_entries
         (vehicle_id, period_year, period_month, fuel_limit, paid_amount, remaining_limit,
          distance_km, current_km, liter_amount, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   (SELECT u.full_name FROM vehicles v LEFT JOIN users u ON u.id = v.user_id WHERE v.id = vehicle_id) AS assigned_user_name,
                   period_year, period_month, fuel_limit, paid_amount, remaining_limit, distance_km,
                   current_km, liter_amount, note, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.period_year)
    .bind(payload.period_month)
    .bind(payload.fuel_limit)
    .bind(payload.paid_amount)
    .bind(payload.remaining_limit)
    .bind(payload.distance_km)
    .bind(payload.current_km)
    .bind(payload.liter_amount)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(&state, &headers, "fuel_entries", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn get_fuel_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<FuelEntryRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_fuel_entry(&state, id).await?))
}

async fn update_fuel_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<FuelEntryUpdate>,
) -> ApiResult<Json<FuelEntryRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_fuel_entry(&state, id).await?;
    let row = sqlx::query_as::<_, FuelEntryRow>(
        "UPDATE fuel_entries SET
            fuel_limit = COALESCE($2, fuel_limit),
            paid_amount = COALESCE($3, paid_amount),
            remaining_limit = COALESCE($4, remaining_limit),
            distance_km = COALESCE($5, distance_km),
            current_km = COALESCE($6, current_km),
            liter_amount = COALESCE($7, liter_amount),
            note = COALESCE($8, note),
            updated_by = $9,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   (SELECT u.full_name FROM vehicles v LEFT JOIN users u ON u.id = v.user_id WHERE v.id = vehicle_id) AS assigned_user_name,
                   period_year, period_month, fuel_limit, paid_amount, remaining_limit, distance_km,
                   current_km, liter_amount, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.fuel_limit)
    .bind(payload.paid_amount)
    .bind(payload.remaining_limit)
    .bind(payload.distance_km)
    .bind(payload.current_km)
    .bind(payload.liter_amount)
    .bind(payload.note)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(&state, &headers, "fuel_entries", id, &old, &row, user.id).await?;
    Ok(Json(row))
}

async fn archive_fuel_entry(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<FuelEntryRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_fuel_entry(&state, id).await?;
    let row = sqlx::query_as::<_, FuelEntryRow>(
        "UPDATE fuel_entries SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   (SELECT u.full_name FROM vehicles v LEFT JOIN users u ON u.id = v.user_id WHERE v.id = vehicle_id) AS assigned_user_name,
                   period_year, period_month, fuel_limit, paid_amount, remaining_limit, distance_km,
                   current_km, liter_amount, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(&state, &headers, "fuel_entries", id, &old, &row, user.id).await?;
    Ok(Json(row))
}

async fn list_vehicle_washes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<VehicleWashRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, VehicleWashRow>(
        "SELECT w.id, w.vehicle_id, v.plate, w.branch, w.wash_company, w.wash_date, w.amount,
                w.user_id, u.full_name AS user_name, w.note, w.created_at, w.updated_at, w.created_by
         FROM vehicle_washes w
         JOIN vehicles v ON v.id = w.vehicle_id
         LEFT JOIN users u ON u.id = w.user_id
         WHERE w.deleted_at IS NULL
           AND ($1::bigint IS NULL OR w.vehicle_id = $1)
         ORDER BY w.wash_date DESC, w.created_at DESC
         LIMIT $2",
    )
    .bind(query.vehicle_id)
    .bind(limit(query.limit))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_vehicle_wash(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<VehicleWashCreate>,
) -> ApiResult<Json<VehicleWashRow>> {
    let user = authorize_write(&state, &headers).await?;
    let row = sqlx::query_as::<_, VehicleWashRow>(
        "INSERT INTO vehicle_washes (vehicle_id, branch, wash_company, wash_date, amount, user_id, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, wash_company, wash_date, amount, user_id,
                   (SELECT full_name FROM users WHERE id = user_id) AS user_name,
                   note, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.branch)
    .bind(payload.wash_company)
    .bind(payload.wash_date)
    .bind(payload.amount)
    .bind(payload.user_id)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(&state, &headers, "vehicle_washes", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn get_vehicle_wash(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleWashRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_vehicle_wash(&state, id).await?))
}

async fn update_vehicle_wash(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<VehicleWashUpdate>,
) -> ApiResult<Json<VehicleWashRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_vehicle_wash(&state, id).await?;
    let row = sqlx::query_as::<_, VehicleWashRow>(
        "UPDATE vehicle_washes SET
            branch = COALESCE($2, branch),
            wash_company = COALESCE($3, wash_company),
            wash_date = COALESCE($4, wash_date),
            amount = COALESCE($5, amount),
            user_id = COALESCE($6, user_id),
            note = COALESCE($7, note),
            updated_by = $8,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, wash_company, wash_date, amount, user_id,
                   (SELECT full_name FROM users WHERE id = user_id) AS user_name,
                   note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.branch)
    .bind(payload.wash_company)
    .bind(payload.wash_date)
    .bind(payload.amount)
    .bind(payload.user_id)
    .bind(payload.note)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(&state, &headers, "vehicle_washes", id, &old, &row, user.id).await?;
    Ok(Json(row))
}

async fn archive_vehicle_wash(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<VehicleWashRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_vehicle_wash(&state, id).await?;
    let row = sqlx::query_as::<_, VehicleWashRow>(
        "UPDATE vehicle_washes SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   branch, wash_company, wash_date, amount, user_id,
                   (SELECT full_name FROM users WHERE id = user_id) AS user_name,
                   note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(&state, &headers, "vehicle_washes", id, &old, &row, user.id).await?;
    Ok(Json(row))
}

async fn list_insurance_quotes(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<Vec<InsuranceQuoteRow>>> {
    authorize_read(&state, &headers).await?;
    let rows = sqlx::query_as::<_, InsuranceQuoteRow>(
        "SELECT q.id, q.vehicle_id, v.plate, q.quote_type, q.insurance_company, q.agency_name,
                q.gross_premium, q.installment_count, q.quote_status, q.valid_until, q.note,
                q.created_at, q.updated_at, q.created_by
         FROM insurance_quotes q
         JOIN vehicles v ON v.id = q.vehicle_id
         WHERE q.deleted_at IS NULL
           AND ($1::bigint IS NULL OR q.vehicle_id = $1)
           AND ($2::text IS NULL OR q.quote_status = $2)
         ORDER BY q.valid_until ASC NULLS LAST, q.created_at DESC
         LIMIT $3",
    )
    .bind(query.vehicle_id)
    .bind(query.status)
    .bind(limit(query.limit))
    .fetch_all(&state.pool)
    .await?;
    Ok(Json(rows))
}

async fn create_insurance_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<InsuranceQuoteCreate>,
) -> ApiResult<Json<InsuranceQuoteRow>> {
    let user = authorize_write(&state, &headers).await?;
    let row = sqlx::query_as::<_, InsuranceQuoteRow>(
        "INSERT INTO insurance_quotes
         (vehicle_id, quote_type, insurance_company, agency_name, gross_premium,
          installment_count, quote_status, valid_until, note, created_by)
         VALUES ($1, COALESCE($2, 'kasko'), $3, $4, $5, $6, COALESCE($7, 'pending'), $8, $9, $10)
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   quote_type, insurance_company, agency_name, gross_premium, installment_count,
                   quote_status, valid_until, note, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.quote_type)
    .bind(payload.insurance_company)
    .bind(payload.agency_name)
    .bind(payload.gross_premium)
    .bind(payload.installment_count)
    .bind(payload.quote_status)
    .bind(payload.valid_until)
    .bind(payload.note)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await?;
    audit_create(&state, &headers, "insurance_quotes", row.id, &row, user.id).await?;
    Ok(Json(row))
}

async fn get_insurance_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<InsuranceQuoteRow>> {
    authorize_read(&state, &headers).await?;
    Ok(Json(find_insurance_quote(&state, id).await?))
}

async fn update_insurance_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<InsuranceQuoteUpdate>,
) -> ApiResult<Json<InsuranceQuoteRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_insurance_quote(&state, id).await?;
    let row = sqlx::query_as::<_, InsuranceQuoteRow>(
        "UPDATE insurance_quotes SET
            quote_type = COALESCE($2, quote_type),
            insurance_company = COALESCE($3, insurance_company),
            agency_name = COALESCE($4, agency_name),
            gross_premium = COALESCE($5, gross_premium),
            installment_count = COALESCE($6, installment_count),
            quote_status = COALESCE($7, quote_status),
            valid_until = COALESCE($8, valid_until),
            note = COALESCE($9, note),
            updated_by = $10,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   quote_type, insurance_company, agency_name, gross_premium, installment_count,
                   quote_status, valid_until, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.quote_type)
    .bind(payload.insurance_company)
    .bind(payload.agency_name)
    .bind(payload.gross_premium)
    .bind(payload.installment_count)
    .bind(payload.quote_status)
    .bind(payload.valid_until)
    .bind(payload.note)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_update(
        &state,
        &headers,
        "insurance_quotes",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn archive_insurance_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<InsuranceQuoteRow>> {
    let user = authorize_write(&state, &headers).await?;
    let old = find_insurance_quote(&state, id).await?;
    let row = sqlx::query_as::<_, InsuranceQuoteRow>(
        "UPDATE insurance_quotes SET deleted_at = now(), is_active = false, updated_by = $2, updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, (SELECT plate FROM vehicles WHERE id = vehicle_id) AS plate,
                   quote_type, insurance_company, agency_name, gross_premium, installment_count,
                   quote_status, valid_until, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;
    audit_delete(
        &state,
        &headers,
        "insurance_quotes",
        id,
        &old,
        &row,
        user.id,
    )
    .await?;
    Ok(Json(row))
}

async fn find_vehicle_inspection(state: &AppState, id: i64) -> ApiResult<VehicleInspectionRow> {
    sqlx::query_as::<_, VehicleInspectionRow>(
        "SELECT i.id, i.vehicle_id, v.plate, i.branch, i.inspection_date, i.inspector_user_id,
                u.full_name AS inspector_name, i.exterior_ok, i.interior_ok, i.equipment_ok, i.documents_ok,
                i.damage_note, i.action_note, i.expert_report, i.fee, i.inspection_status,
                i.created_at, i.updated_at, i.created_by
         FROM vehicle_inspections i
         JOIN vehicles v ON v.id = i.vehicle_id
         LEFT JOIN users u ON u.id = i.inspector_user_id
         WHERE i.id = $1 AND i.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_value_loss_claim(state: &AppState, id: i64) -> ApiResult<ValueLossClaimRow> {
    sqlx::query_as::<_, ValueLossClaimRow>(
        "SELECT c.id, c.vehicle_id, v.plate, c.accident_date, c.vehicle_purchase_date, c.tramer_amount,
                c.deprivation_days, c.requested_amount, c.received_amount, c.claim_status, c.note,
                c.created_at, c.updated_at, c.created_by
         FROM value_loss_claims c
         JOIN vehicles v ON v.id = c.vehicle_id
         WHERE c.id = $1 AND c.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_fuel_entry(state: &AppState, id: i64) -> ApiResult<FuelEntryRow> {
    sqlx::query_as::<_, FuelEntryRow>(
        "SELECT f.id, f.vehicle_id, v.plate, u.full_name AS assigned_user_name, f.period_year, f.period_month,
                f.fuel_limit, f.paid_amount, f.remaining_limit, f.distance_km, f.current_km,
                f.liter_amount, f.note, f.created_at, f.updated_at, f.created_by
         FROM fuel_entries f
         JOIN vehicles v ON v.id = f.vehicle_id
         LEFT JOIN users u ON u.id = v.user_id
         WHERE f.id = $1 AND f.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_wash(state: &AppState, id: i64) -> ApiResult<VehicleWashRow> {
    sqlx::query_as::<_, VehicleWashRow>(
        "SELECT w.id, w.vehicle_id, v.plate, w.branch, w.wash_company, w.wash_date, w.amount,
                w.user_id, u.full_name AS user_name, w.note, w.created_at, w.updated_at, w.created_by
         FROM vehicle_washes w
         JOIN vehicles v ON v.id = w.vehicle_id
         LEFT JOIN users u ON u.id = w.user_id
         WHERE w.id = $1 AND w.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_insurance_quote(state: &AppState, id: i64) -> ApiResult<InsuranceQuoteRow> {
    sqlx::query_as::<_, InsuranceQuoteRow>(
        "SELECT q.id, q.vehicle_id, v.plate, q.quote_type, q.insurance_company, q.agency_name,
                q.gross_premium, q.installment_count, q.quote_status, q.valid_until, q.note,
                q.created_at, q.updated_at, q.created_by
         FROM insurance_quotes q
         JOIN vehicles v ON v.id = q.vehicle_id
         WHERE q.id = $1 AND q.deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn authorize_read(state: &AppState, headers: &HeaderMap) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation", "accounting"])?;
    Ok(user)
}

async fn authorize_write(state: &AppState, headers: &HeaderMap) -> ApiResult<auth::AuthUser> {
    let user = auth::current_user(state, headers).await?;
    auth::require_roles(&user, &["admin", "manager", "operation"])?;
    Ok(user)
}

fn limit(value: Option<i64>) -> i64 {
    value.unwrap_or(100).clamp(1, 500)
}

async fn audit_create<T: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    row: &T,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "create",
        None,
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}

async fn audit_update<T: Serialize, U: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    old: &T,
    row: &U,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "update",
        Some(serde_json::to_value(old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}

async fn audit_delete<T: Serialize, U: Serialize>(
    state: &AppState,
    headers: &HeaderMap,
    table: &str,
    id: i64,
    old: &T,
    row: &U,
    user_id: i64,
) -> ApiResult<()> {
    audit::write_audit(
        &state.pool,
        table,
        Some(id),
        "delete",
        Some(serde_json::to_value(old).unwrap_or_else(|_| json!({}))),
        serde_json::to_value(row).unwrap_or_else(|_| json!({})),
        Some(user_id),
        headers,
    )
    .await
}
