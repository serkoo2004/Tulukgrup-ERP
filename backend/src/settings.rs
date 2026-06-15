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
use serde_json::{json, Value};
use sqlx::{types::Json as SqlJson, FromRow};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_settings).post(upsert_setting))
        .route("/:key", get(get_setting).patch(update_setting))
}

#[derive(Debug, Deserialize)]
struct SettingsQuery {
    active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SettingUpsert {
    setting_key: String,
    setting_value: Value,
    description: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SettingUpdate {
    setting_value: Option<Value>,
    description: Option<String>,
    is_active: Option<bool>,
}

#[derive(Debug, FromRow, Serialize)]
struct SettingRow {
    id: i64,
    setting_key: String,
    setting_value: Value,
    description: Option<String>,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

async fn list_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SettingsQuery>,
) -> ApiResult<Json<Vec<SettingRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;

    let rows = sqlx::query_as::<_, SettingRow>(
        "SELECT id, setting_key, setting_value, description, is_active,
                created_at, updated_at, created_by
         FROM system_settings
         WHERE deleted_at IS NULL
           AND ($1::boolean IS NULL OR is_active = $1)
         ORDER BY setting_key",
    )
    .bind(query.active)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn upsert_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SettingUpsert>,
) -> ApiResult<Json<SettingRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin"])?;
    validate_setting_key(&payload.setting_key)?;

    let row = sqlx::query_as::<_, SettingRow>(
        "INSERT INTO system_settings
         (setting_key, setting_value, description, is_active, created_by, updated_by)
         VALUES ($1, $2, $3, COALESCE($4, true), $5, $5)
         ON CONFLICT (setting_key) DO UPDATE SET
            setting_value = EXCLUDED.setting_value,
            description = EXCLUDED.description,
            is_active = EXCLUDED.is_active,
            updated_by = EXCLUDED.updated_by,
            updated_at = now(),
            deleted_at = NULL
         RETURNING id, setting_key, setting_value, description, is_active,
                   created_at, updated_at, created_by",
    )
    .bind(payload.setting_key)
    .bind(SqlJson(payload.setting_value))
    .bind(payload.description)
    .bind(payload.is_active)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "system_settings",
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

async fn get_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key): Path<String>,
) -> ApiResult<Json<SettingRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    Ok(Json(find_setting(&state.pool, &key).await?))
}

async fn update_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(key): Path<String>,
    Json(payload): Json<SettingUpdate>,
) -> ApiResult<Json<SettingRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin"])?;
    let old = find_setting(&state.pool, &key).await?;

    let row = sqlx::query_as::<_, SettingRow>(
        "UPDATE system_settings SET
            setting_value = COALESCE($2::jsonb, setting_value),
            description = COALESCE($3, description),
            is_active = COALESCE($4, is_active),
            updated_by = $5,
            updated_at = now()
         WHERE setting_key = $1 AND deleted_at IS NULL
         RETURNING id, setting_key, setting_value, description, is_active,
                   created_at, updated_at, created_by",
    )
    .bind(key)
    .bind(payload.setting_value.map(SqlJson))
    .bind(payload.description)
    .bind(payload.is_active)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "system_settings",
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

async fn find_setting(pool: &sqlx::PgPool, key: &str) -> ApiResult<SettingRow> {
    sqlx::query_as::<_, SettingRow>(
        "SELECT id, setting_key, setting_value, description, is_active,
                created_at, updated_at, created_by
         FROM system_settings
         WHERE setting_key = $1 AND deleted_at IS NULL",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn validate_setting_key(key: &str) -> ApiResult<()> {
    let is_valid = !key.trim().is_empty()
        && key.len() <= 160
        && key.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
        });

    if is_valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "setting_key must use lowercase letters, numbers, dot, underscore or dash".to_string(),
        ))
    }
}
