use crate::{auth, error::ApiResult, AppState};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{types::Json as SqlJson, FromRow, PgPool};

pub fn router() -> Router<AppState> {
    Router::new().route("/audit", get(list_audit_logs))
}

#[derive(Debug, Deserialize)]
struct AuditQuery {
    table_name: Option<String>,
    record_id: Option<i64>,
    action_type: Option<String>,
    created_by: Option<i64>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    limit: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct AuditLogRow {
    id: i64,
    table_name: String,
    record_id: Option<i64>,
    action_type: String,
    old_data: Option<Value>,
    new_data: Option<Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_by: Option<i64>,
    created_at: DateTime<Utc>,
}

async fn list_audit_logs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditQuery>,
) -> ApiResult<Json<Vec<AuditLogRow>>> {
    let user = auth::current_user(&state, &headers).await?;
    auth::require_roles(&user, &["admin", "manager"])?;

    let rows = sqlx::query_as::<_, AuditLogRow>(
        "SELECT id, table_name, record_id, action_type::text AS action_type,
                old_data, new_data, ip_address::text AS ip_address, user_agent, created_by, created_at
         FROM audit_logs
         WHERE ($1::text IS NULL OR table_name = $1)
           AND ($2::bigint IS NULL OR record_id = $2)
           AND ($3::text IS NULL OR action_type::text = $3)
           AND ($4::bigint IS NULL OR created_by = $4)
           AND ($5::date IS NULL OR created_at >= $5)
           AND ($6::date IS NULL OR created_at < ($6 + INTERVAL '1 day'))
         ORDER BY created_at DESC
         LIMIT $7",
    )
    .bind(query.table_name)
    .bind(query.record_id)
    .bind(query.action_type)
    .bind(query.created_by)
    .bind(query.start_date)
    .bind(query.end_date)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

pub async fn write_audit(
    pool: &PgPool,
    table_name: &str,
    record_id: Option<i64>,
    action_type: &str,
    old_data: Option<Value>,
    new_data: Value,
    actor_id: Option<i64>,
    headers: &HeaderMap,
) -> ApiResult<()> {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    sqlx::query(
        "INSERT INTO audit_logs
         (table_name, record_id, action_type, old_data, new_data, user_agent, created_by)
         VALUES ($1, $2, $3::audit_action, $4, $5, $6, $7)",
    )
    .bind(table_name)
    .bind(record_id)
    .bind(action_type)
    .bind(old_data.map(SqlJson))
    .bind(SqlJson(new_data))
    .bind(user_agent)
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}
