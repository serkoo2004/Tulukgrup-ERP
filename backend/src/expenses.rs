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
        .route("/", get(list_expenses).post(create_expense))
        .route(
            "/:id",
            get(get_expense)
                .patch(update_payment_status)
                .delete(archive_expense),
        )
}

#[derive(Debug, Deserialize)]
struct ExpenseQuery {
    vehicle_id: Option<i64>,
    expense_type: Option<String>,
    payment_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ExpenseCreate {
    vehicle_id: i64,
    expense_type: String,
    amount: Decimal,
    invoice_number: Option<String>,
    invoice_file: Option<String>,
    payment_status: Option<String>,
    expense_date: NaiveDate,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PaymentStatusUpdate {
    payment_status: String,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseRow {
    id: i64,
    vehicle_id: i64,
    expense_type: String,
    amount: Decimal,
    invoice_number: Option<String>,
    invoice_file: Option<String>,
    payment_status: Option<String>,
    expense_date: NaiveDate,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ExpenseDetail {
    expense: ExpenseRow,
    file_summary: ExpenseFileSummary,
}

#[derive(Debug, FromRow, Serialize)]
struct ExpenseFileSummary {
    total_files: i64,
    invoice_files: i64,
    payment_receipts: i64,
    expense_documents: i64,
}

#[derive(Debug, FromRow)]
struct VehicleAccessRow {
    status: String,
    is_active: bool,
}

async fn list_expenses(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ExpenseQuery>,
) -> ApiResult<Json<Vec<ExpenseRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, ExpenseRow>(
        "SELECT id, vehicle_id, expense_type::text AS expense_type, amount, invoice_number,
                invoice_file, payment_status, expense_date, note, created_at, updated_at, created_by
         FROM expenses
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR vehicle_id = $1)
           AND ($2::text IS NULL OR expense_type::text = $2)
           AND ($3::text IS NULL OR payment_status = $3)
         ORDER BY expense_date DESC, created_at DESC
         LIMIT $4",
    )
    .bind(query.vehicle_id)
    .bind(query.expense_type)
    .bind(query.payment_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_expense(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ExpenseCreate>,
) -> ApiResult<Json<ExpenseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    validate_expense_type(&payload.expense_type)?;
    validate_payment_status_optional(payload.payment_status.as_deref())?;
    let vehicle = find_vehicle_for_expense(&state.pool, payload.vehicle_id).await?;
    ensure_vehicle_allows_expense(&vehicle)?;

    let row = sqlx::query_as::<_, ExpenseRow>(
        "INSERT INTO expenses
         (vehicle_id, expense_type, amount, invoice_number, invoice_file,
          payment_status, expense_date, note, created_by)
         VALUES ($1, $2::expense_type, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, vehicle_id, expense_type::text AS expense_type, amount, invoice_number,
                   invoice_file, payment_status, expense_date, note, created_at, updated_at, created_by",
    )
    .bind(payload.vehicle_id)
    .bind(payload.expense_type)
    .bind(payload.amount)
    .bind(payload.invoice_number)
    .bind(payload.invoice_file)
    .bind(payload.payment_status)
    .bind(payload.expense_date)
    .bind(payload.note)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "expenses",
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

async fn get_expense(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ExpenseDetail>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    let expense = find_expense(&state.pool, id).await?;
    let file_summary = expense_file_summary(&state.pool, id).await?;

    Ok(Json(ExpenseDetail {
        expense,
        file_summary,
    }))
}

async fn update_payment_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<PaymentStatusUpdate>,
) -> ApiResult<Json<ExpenseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "accounting"])?;
    validate_payment_status_optional(Some(&payload.payment_status))?;
    let old = find_expense(&state.pool, id).await?;

    let row = sqlx::query_as::<_, ExpenseRow>(
        "UPDATE expenses
         SET payment_status = $2,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, expense_type::text AS expense_type, amount, invoice_number,
                   invoice_file, payment_status, expense_date, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.payment_status)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "expenses",
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

async fn archive_expense(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ExpenseRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "accounting"])?;
    let old = find_expense(&state.pool, id).await?;

    let row = sqlx::query_as::<_, ExpenseRow>(
        "UPDATE expenses
         SET deleted_at = now(),
             is_active = false,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, vehicle_id, expense_type::text AS expense_type, amount, invoice_number,
                   invoice_file, payment_status, expense_date, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "expenses",
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

async fn find_expense(pool: &PgPool, id: i64) -> ApiResult<ExpenseRow> {
    sqlx::query_as::<_, ExpenseRow>(
        "SELECT id, vehicle_id, expense_type::text AS expense_type, amount, invoice_number,
                invoice_file, payment_status, expense_date, note, created_at, updated_at, created_by
         FROM expenses
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_vehicle_for_expense(pool: &PgPool, vehicle_id: i64) -> ApiResult<VehicleAccessRow> {
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

async fn expense_file_summary(pool: &PgPool, expense_id: i64) -> ApiResult<ExpenseFileSummary> {
    Ok(sqlx::query_as::<_, ExpenseFileSummary>(
        "SELECT
            count(*) AS total_files,
            count(*) FILTER (WHERE file_type = 'invoice') AS invoice_files,
            count(*) FILTER (WHERE file_type = 'payment_receipt') AS payment_receipts,
            count(*) FILTER (WHERE file_type = 'expense_document') AS expense_documents
         FROM file_documents
         WHERE module_name = 'expenses'
           AND entity_id = $1
           AND deleted_at IS NULL",
    )
    .bind(expense_id)
    .fetch_one(pool)
    .await?)
}

fn ensure_vehicle_allows_expense(vehicle: &VehicleAccessRow) -> ApiResult<()> {
    if vehicle.status == "sold" || !vehicle.is_active {
        Err(ApiError::Conflict(
            "passive or sold vehicles cannot receive new expense records".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn validate_expense_type(expense_type: &str) -> ApiResult<()> {
    match expense_type {
        "fuel" | "insurance" | "maintenance" | "tax" | "fine" | "tire" | "service" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid expense type".to_string())),
    }
}

fn validate_payment_status_optional(status: Option<&str>) -> ApiResult<()> {
    match status {
        None | Some("pending") | Some("paid") | Some("cancelled") | Some("overdue") => Ok(()),
        _ => Err(ApiError::BadRequest("invalid payment status".to_string())),
    }
}
