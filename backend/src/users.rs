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
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/roles", get(list_roles))
        .route(
            "/:id/mobile-permissions",
            get(list_mobile_permissions).post(upsert_mobile_permission),
        )
        .route(
            "/:id",
            get(get_user).patch(update_user).delete(deactivate_user),
        )
}

#[derive(Debug, Deserialize)]
struct UserQuery {
    role: Option<String>,
    q: Option<String>,
    include_inactive: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct UserCreate {
    email: String,
    full_name: String,
    password: String,
    role: String,
    phone: Option<String>,
    extension: Option<String>,
    company_id: Option<i64>,
    department_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct UserUpdate {
    full_name: Option<String>,
    role: Option<String>,
    phone: Option<String>,
    extension: Option<String>,
    company_id: Option<i64>,
    department_id: Option<i64>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct MobilePermissionUpsert {
    company_id: Option<i64>,
    department_id: Option<i64>,
    permission_key: String,
    can_view: Option<bool>,
    can_create: Option<bool>,
    can_update: Option<bool>,
    can_approve: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct UserRow {
    id: i64,
    email: String,
    full_name: String,
    role: String,
    phone: Option<String>,
    extension: Option<String>,
    company_id: Option<i64>,
    department_id: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_active: bool,
}

#[derive(Debug, Serialize)]
struct RoleRow {
    key: &'static str,
    label: &'static str,
}

#[derive(Debug, FromRow, Serialize)]
struct MobilePermissionRow {
    id: i64,
    user_id: i64,
    company_id: Option<i64>,
    company_name: Option<String>,
    department_id: Option<i64>,
    department_name: Option<String>,
    permission_key: String,
    can_view: bool,
    can_create: bool,
    can_update: bool,
    can_approve: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<UserQuery>,
) -> ApiResult<Json<Vec<UserRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let q = query.q.map(|value| format!("%{}%", value));
    let rows = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, full_name, role::text AS role, phone, extension,
                company_id, department_id, created_at, updated_at, is_active
         FROM users
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR role::text = $1)
           AND ($2::text IS NULL OR email ILIKE $2 OR full_name ILIKE $2)
           AND ($3::boolean = true OR is_active = true)
         ORDER BY full_name",
    )
    .bind(query.role)
    .bind(q)
    .bind(query.include_inactive.unwrap_or(false))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UserCreate>,
) -> ApiResult<Json<UserRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin"])?;
    validate_role(&payload.role)?;

    let password_hash =
        hash(payload.password, DEFAULT_COST).map_err(|err| ApiError::Internal(err.into()))?;
    let row = sqlx::query_as::<_, UserRow>(
        "INSERT INTO users
         (email, full_name, password_hash, role, phone, extension, company_id, department_id, created_by)
         VALUES ($1, $2, $3, $4::user_role, $5, $6, $7, $8, $9)
         RETURNING id, email, full_name, role::text AS role, phone, extension,
                   company_id, department_id, created_at, updated_at, is_active",
    )
    .bind(payload.email.trim().to_lowercase())
    .bind(payload.full_name)
    .bind(password_hash)
    .bind(payload.role)
    .bind(payload.phone)
    .bind(payload.extension)
    .bind(payload.company_id)
    .bind(payload.department_id)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await
    .map_err(map_unique_user_error)?;

    audit::write_audit(
        &state.pool,
        "users",
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

async fn get_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<UserRow>> {
    let current = auth::current_user(&state, &headers).await?;
    if current.id != id {
        auth::require_roles(&current, &["admin", "manager"])?;
    }

    let row = sqlx::query_as::<_, UserRow>(
        "SELECT id, email, full_name, role::text AS role, phone, extension,
                company_id, department_id, created_at, updated_at, is_active
         FROM users
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(row))
}

async fn update_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<UserUpdate>,
) -> ApiResult<Json<UserRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin"])?;
    if let Some(role) = payload.role.as_deref() {
        validate_role(role)?;
    }

    let old = find_user(&state.pool, id).await?;
    let row = sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            full_name = COALESCE($2, full_name),
            role = COALESCE($3::user_role, role),
            phone = COALESCE($4, phone),
            extension = COALESCE($5, extension),
            company_id = COALESCE($6, company_id),
            department_id = COALESCE($7, department_id),
            is_active = COALESCE($8, is_active),
            updated_by = $9,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, email, full_name, role::text AS role, phone, extension,
                   company_id, department_id, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(payload.full_name)
    .bind(payload.role)
    .bind(payload.phone)
    .bind(payload.extension)
    .bind(payload.company_id)
    .bind(payload.department_id)
    .bind(payload.is_active)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "users",
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

async fn deactivate_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<UserRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin"])?;
    if current.id == id {
        return Err(ApiError::Conflict(
            "current user cannot deactivate itself".to_string(),
        ));
    }
    let old = find_user(&state.pool, id).await?;

    let mut tx = state.pool.begin().await?;
    let row = sqlx::query_as::<_, UserRow>(
        "UPDATE users SET
            is_active = false,
            deleted_at = now(),
            updated_by = $2,
            updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, email, full_name, role::text AS role, phone, extension,
                   company_id, department_id, created_at, updated_at, is_active",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ApiError::NotFound)?;

    sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    audit::write_audit(
        &state.pool,
        "users",
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

async fn list_mobile_permissions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<MobilePermissionRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let _ = find_user(&state.pool, id).await?;

    let rows = sqlx::query_as::<_, MobilePermissionRow>(
        "SELECT p.id, p.user_id, p.company_id, c.name AS company_name,
                p.department_id, d.name AS department_name, p.permission_key,
                p.can_view, p.can_create, p.can_update, p.can_approve,
                p.created_at, p.updated_at, p.created_by
         FROM mobile_user_permissions p
         LEFT JOIN companies c ON c.id = p.company_id
         LEFT JOIN departments d ON d.id = p.department_id
         WHERE p.user_id = $1 AND p.deleted_at IS NULL
         ORDER BY c.name NULLS FIRST, d.name NULLS FIRST, p.permission_key",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn upsert_mobile_permission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<MobilePermissionUpsert>,
) -> ApiResult<Json<MobilePermissionRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let _ = find_user(&state.pool, id).await?;
    validate_mobile_permission_key(&payload.permission_key)?;

    let existing_id: Option<i64> = sqlx::query_scalar(
        "SELECT id
         FROM mobile_user_permissions
         WHERE user_id = $1
           AND COALESCE(company_id, 0) = COALESCE($2, 0)
           AND COALESCE(department_id, 0) = COALESCE($3, 0)
           AND permission_key = $4",
    )
    .bind(id)
    .bind(payload.company_id)
    .bind(payload.department_id)
    .bind(&payload.permission_key)
    .fetch_optional(&state.pool)
    .await?;

    let row = if let Some(permission_id) = existing_id {
        sqlx::query_as::<_, MobilePermissionRow>(
            "WITH updated AS (
                UPDATE mobile_user_permissions
                SET can_view = COALESCE($2, can_view),
                    can_create = COALESCE($3, can_create),
                    can_update = COALESCE($4, can_update),
                    can_approve = COALESCE($5, can_approve),
                    deleted_at = NULL,
                    is_active = true,
                    updated_by = $6,
                    updated_at = now()
                WHERE id = $1
                RETURNING *
             )
             SELECT p.id, p.user_id, p.company_id, c.name AS company_name,
                    p.department_id, d.name AS department_name, p.permission_key,
                    p.can_view, p.can_create, p.can_update, p.can_approve,
                    p.created_at, p.updated_at, p.created_by
             FROM updated p
             LEFT JOIN companies c ON c.id = p.company_id
             LEFT JOIN departments d ON d.id = p.department_id",
        )
        .bind(permission_id)
        .bind(payload.can_view)
        .bind(payload.can_create)
        .bind(payload.can_update)
        .bind(payload.can_approve)
        .bind(current.id)
        .fetch_one(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, MobilePermissionRow>(
            "WITH inserted AS (
                INSERT INTO mobile_user_permissions
                  (user_id, company_id, department_id, permission_key, can_view,
                   can_create, can_update, can_approve, created_by, updated_by)
                VALUES ($1, $2, $3, $4, COALESCE($5, true), COALESCE($6, false),
                        COALESCE($7, false), COALESCE($8, false), $9, $9)
                RETURNING *
             )
             SELECT p.id, p.user_id, p.company_id, c.name AS company_name,
                    p.department_id, d.name AS department_name, p.permission_key,
                    p.can_view, p.can_create, p.can_update, p.can_approve,
                    p.created_at, p.updated_at, p.created_by
             FROM inserted p
             LEFT JOIN companies c ON c.id = p.company_id
             LEFT JOIN departments d ON d.id = p.department_id",
        )
        .bind(id)
        .bind(payload.company_id)
        .bind(payload.department_id)
        .bind(&payload.permission_key)
        .bind(payload.can_view)
        .bind(payload.can_create)
        .bind(payload.can_update)
        .bind(payload.can_approve)
        .bind(current.id)
        .fetch_one(&state.pool)
        .await?
    };

    audit::write_audit(
        &state.pool,
        "mobile_user_permissions",
        Some(row.id),
        "update",
        None,
        serde_json::to_value(&row).unwrap_or_else(|_| json!({})),
        Some(current.id),
        &headers,
    )
    .await?;

    Ok(Json(row))
}

async fn list_roles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<RoleRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    Ok(Json(vec![
        RoleRow {
            key: "admin",
            label: "Admin",
        },
        RoleRow {
            key: "manager",
            label: "Yonetici",
        },
        RoleRow {
            key: "operation",
            label: "Operasyon",
        },
        RoleRow {
            key: "accounting",
            label: "Muhasebe",
        },
        RoleRow {
            key: "user",
            label: "Kullanici",
        },
    ]))
}

async fn find_user(pool: &sqlx::PgPool, id: i64) -> ApiResult<UserRow> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, email, full_name, role::text AS role, phone, extension,
                company_id, department_id, created_at, updated_at, is_active
         FROM users
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn validate_role(role: &str) -> ApiResult<()> {
    match role {
        "admin" | "manager" | "operation" | "accounting" | "user" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid role".to_string())),
    }
}

fn validate_mobile_permission_key(key: &str) -> ApiResult<()> {
    match key {
        "dashboard" | "inventory_view" | "inventory_order" | "vehicle_view" | "vehicle_fault"
        | "km_log" | "support_ticket" | "media_upload" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid mobile permission key".to_string(),
        )),
    }
}

fn map_unique_user_error(err: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.constraint() == Some("users_email_key") {
            return ApiError::Conflict("email already exists".to_string());
        }
    }
    ApiError::Database(err)
}
