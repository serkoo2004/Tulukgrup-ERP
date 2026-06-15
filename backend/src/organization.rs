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
use serde_json::json;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/companies", get(list_companies).post(create_company))
        .route("/companies/:id", get(get_company).patch(update_company))
        .route(
            "/departments",
            get(list_departments).post(create_department),
        )
        .route(
            "/departments/:id",
            get(get_department).patch(update_department),
        )
}

#[derive(Debug, Deserialize)]
struct CompanyQuery {
    q: Option<String>,
    include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CompanyCreate {
    name: String,
    tax_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CompanyUpdate {
    name: Option<String>,
    tax_number: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct CompanyRow {
    id: i64,
    name: String,
    tax_number: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
}

#[derive(Debug, Deserialize)]
struct DepartmentQuery {
    company_id: Option<i64>,
    q: Option<String>,
    include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DepartmentCreate {
    company_id: Option<i64>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct DepartmentUpdate {
    company_id: Option<i64>,
    name: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct DepartmentRow {
    id: i64,
    company_id: Option<i64>,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
}

async fn list_companies(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CompanyQuery>,
) -> ApiResult<Json<Vec<CompanyRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    let q = query.q.map(|value| format!("%{}%", value));

    let rows = sqlx::query_as::<_, CompanyRow>(
        "SELECT id, name, tax_number, created_at, updated_at, is_active
         FROM companies
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR name ILIKE $1 OR tax_number ILIKE $1)
           AND ($2::boolean = true OR is_active = true)
         ORDER BY name",
    )
    .bind(q)
    .bind(query.include_inactive.unwrap_or(false))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_company(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CompanyCreate>,
) -> ApiResult<Json<CompanyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let row = sqlx::query_as::<_, CompanyRow>(
        "INSERT INTO companies (name, tax_number, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, name, tax_number, created_at, updated_at, is_active",
    )
    .bind(payload.name)
    .bind(payload.tax_number)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_unique_company_error)?;

    audit::write_audit(
        &state.pool,
        "companies",
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

async fn get_company(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<CompanyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_company(&state.pool, id).await?))
}

async fn update_company(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<CompanyUpdate>,
) -> ApiResult<Json<CompanyRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_company(&state.pool, id).await?;

    let row = sqlx::query_as::<_, CompanyRow>(
        "UPDATE companies
         SET name = COALESCE($2, name),
             tax_number = COALESCE($3, tax_number),
             is_active = COALESCE($4, is_active),
             updated_by = $5,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, name, tax_number, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(payload.name)
    .bind(payload.tax_number)
    .bind(payload.is_active)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "companies",
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

async fn list_departments(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DepartmentQuery>,
) -> ApiResult<Json<Vec<DepartmentRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    let q = query.q.map(|value| format!("%{}%", value));

    let rows = sqlx::query_as::<_, DepartmentRow>(
        "SELECT id, company_id, name, created_at, updated_at, is_active
         FROM departments
         WHERE deleted_at IS NULL
           AND ($1::bigint IS NULL OR company_id = $1)
           AND ($2::text IS NULL OR name ILIKE $2)
           AND ($3::boolean = true OR is_active = true)
         ORDER BY name",
    )
    .bind(query.company_id)
    .bind(q)
    .bind(query.include_inactive.unwrap_or(false))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_department(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DepartmentCreate>,
) -> ApiResult<Json<DepartmentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let row = sqlx::query_as::<_, DepartmentRow>(
        "INSERT INTO departments (company_id, name, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, company_id, name, created_at, updated_at, is_active",
    )
    .bind(payload.company_id)
    .bind(payload.name)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_unique_department_error)?;

    audit::write_audit(
        &state.pool,
        "departments",
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

async fn get_department(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<DepartmentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_department(&state.pool, id).await?))
}

async fn update_department(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<DepartmentUpdate>,
) -> ApiResult<Json<DepartmentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_department(&state.pool, id).await?;

    let row = sqlx::query_as::<_, DepartmentRow>(
        "UPDATE departments
         SET company_id = COALESCE($2, company_id),
             name = COALESCE($3, name),
             is_active = COALESCE($4, is_active),
             updated_by = $5,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, company_id, name, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(payload.company_id)
    .bind(payload.name)
    .bind(payload.is_active)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "departments",
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

async fn find_company(pool: &sqlx::PgPool, id: i64) -> ApiResult<CompanyRow> {
    sqlx::query_as::<_, CompanyRow>(
        "SELECT id, name, tax_number, created_at, updated_at, is_active
         FROM companies
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn find_department(pool: &sqlx::PgPool, id: i64) -> ApiResult<DepartmentRow> {
    sqlx::query_as::<_, DepartmentRow>(
        "SELECT id, company_id, name, created_at, updated_at, is_active
         FROM departments
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn map_unique_company_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.constraint() == Some("companies_name_key") {
            return ApiError::Conflict("company name already exists".to_string());
        }
    }
    ApiError::Database(err)
}

fn map_unique_department_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.constraint() == Some("uq_department_company_name") {
            return ApiError::Conflict("department already exists for company".to_string());
        }
    }
    ApiError::Database(err)
}
