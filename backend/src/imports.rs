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
use sqlx::{types::Json as SqlJson, FromRow};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/jobs", get(list_import_jobs).post(create_import_job))
        .route(
            "/jobs/:id",
            get(get_import_job).patch(update_import_job_status),
        )
        .route(
            "/jobs/:id/errors",
            get(list_import_errors).post(add_import_error),
        )
        .route(
            "/jobs/:id/rows",
            get(list_import_rows).post(upsert_import_rows),
        )
        .route("/jobs/:id/validate", post(validate_import_job))
}

#[derive(Debug, Deserialize)]
struct ImportJobQuery {
    target_module: Option<String>,
    import_status: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ImportJobCreate {
    target_module: String,
    source_file_id: Option<i64>,
    column_mapping: Option<Value>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImportJobStatusUpdate {
    import_status: String,
    total_rows: Option<i32>,
    processed_rows: Option<i32>,
    error_count: Option<i32>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ImportErrorCreate {
    row_number: Option<i32>,
    field_name: Option<String>,
    error_message: String,
    raw_data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ImportRowsPayload {
    rows: Vec<Value>,
}

#[derive(Debug, FromRow, Serialize)]
struct ImportJobRow {
    id: i64,
    target_module: String,
    source_file_id: Option<i64>,
    column_mapping: Option<Value>,
    import_status: String,
    total_rows: i32,
    processed_rows: i32,
    error_count: i32,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct ImportErrorRow {
    id: i64,
    import_job_id: i64,
    row_number: Option<i32>,
    field_name: Option<String>,
    error_message: String,
    raw_data: Option<Value>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize)]
struct ImportStagingRow {
    id: i64,
    import_job_id: i64,
    row_number: i32,
    raw_data: Value,
    normalized_data: Option<Value>,
    row_status: String,
    error_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ImportValidationSummary {
    import_job_id: i64,
    total_rows: i32,
    valid_rows: i32,
    invalid_rows: i32,
    error_count: i32,
}

#[derive(Debug)]
struct RowValidationError {
    field_name: Option<String>,
    message: String,
}

async fn list_import_jobs(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ImportJobQuery>,
) -> ApiResult<Json<Vec<ImportJobRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;

    let rows = sqlx::query_as::<_, ImportJobRow>(
        "SELECT id, target_module, source_file_id, column_mapping, import_status,
                total_rows, processed_rows, error_count, note, created_at, updated_at, created_by
         FROM import_jobs
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR target_module = $1)
           AND ($2::text IS NULL OR import_status = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(query.target_module)
    .bind(query.import_status)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn create_import_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ImportJobCreate>,
) -> ApiResult<Json<ImportJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_target_module(&payload.target_module)?;

    let row = sqlx::query_as::<_, ImportJobRow>(
        "INSERT INTO import_jobs
         (target_module, source_file_id, column_mapping, note, created_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, target_module, source_file_id, column_mapping, import_status,
                   total_rows, processed_rows, error_count, note, created_at, updated_at, created_by",
    )
    .bind(payload.target_module)
    .bind(payload.source_file_id)
    .bind(payload.column_mapping.map(SqlJson))
    .bind(payload.note)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    audit::write_audit(
        &state.pool,
        "import_jobs",
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

async fn get_import_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ImportJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    Ok(Json(find_import_job(&state.pool, id).await?))
}

async fn update_import_job_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ImportJobStatusUpdate>,
) -> ApiResult<Json<ImportJobRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    validate_import_status(&payload.import_status)?;
    let old = find_import_job(&state.pool, id).await?;

    let row = sqlx::query_as::<_, ImportJobRow>(
        "UPDATE import_jobs
         SET import_status = $2,
             total_rows = COALESCE($3, total_rows),
             processed_rows = COALESCE($4, processed_rows),
             error_count = COALESCE($5, error_count),
             note = COALESCE($6, note),
             updated_by = $7,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, target_module, source_file_id, column_mapping, import_status,
                   total_rows, processed_rows, error_count, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(payload.import_status)
    .bind(payload.total_rows)
    .bind(payload.processed_rows)
    .bind(payload.error_count)
    .bind(payload.note)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "import_jobs",
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

async fn list_import_errors(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<ImportErrorRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let _job = find_import_job(&state.pool, id).await?;

    let rows = sqlx::query_as::<_, ImportErrorRow>(
        "SELECT id, import_job_id, row_number, field_name, error_message, raw_data, created_at
         FROM import_errors
         WHERE import_job_id = $1
         ORDER BY row_number ASC NULLS LAST, id ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn add_import_error(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ImportErrorCreate>,
) -> ApiResult<Json<ImportErrorRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let _job = find_import_job(&state.pool, id).await?;

    let row = sqlx::query_as::<_, ImportErrorRow>(
        "INSERT INTO import_errors
         (import_job_id, row_number, field_name, error_message, raw_data)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, import_job_id, row_number, field_name, error_message, raw_data, created_at",
    )
    .bind(id)
    .bind(payload.row_number)
    .bind(payload.field_name)
    .bind(payload.error_message)
    .bind(payload.raw_data.map(SqlJson))
    .fetch_one(&state.pool)
    .await?;

    sqlx::query(
        "UPDATE import_jobs
         SET error_count = error_count + 1,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(current.id)
    .execute(&state.pool)
    .await?;

    Ok(Json(row))
}

async fn list_import_rows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<Vec<ImportStagingRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let _job = find_import_job(&state.pool, id).await?;

    let rows = sqlx::query_as::<_, ImportStagingRow>(
        "SELECT id, import_job_id, row_number, raw_data, normalized_data,
                row_status, error_count, created_at, updated_at
         FROM import_rows
         WHERE import_job_id = $1
         ORDER BY row_number ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn upsert_import_rows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(payload): Json<ImportRowsPayload>,
) -> ApiResult<Json<Vec<ImportStagingRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    if payload.rows.is_empty() {
        return Err(ApiError::BadRequest("rows cannot be empty".to_string()));
    }
    if payload.rows.len() > 5000 {
        return Err(ApiError::BadRequest(
            "a single import job can stage at most 5000 rows".to_string(),
        ));
    }
    let total_rows = payload.rows.len() as i32;
    let _job = find_import_job(&state.pool, id).await?;

    let mut tx = state.pool.begin().await?;
    for (index, raw_data) in payload.rows.into_iter().enumerate() {
        let row_number = (index + 1) as i32;
        sqlx::query(
            "INSERT INTO import_rows (import_job_id, row_number, raw_data)
             VALUES ($1, $2, $3)
             ON CONFLICT (import_job_id, row_number) DO UPDATE SET
                raw_data = EXCLUDED.raw_data,
                normalized_data = NULL,
                row_status = 'pending',
                error_count = 0,
                updated_at = now()",
        )
        .bind(id)
        .bind(row_number)
        .bind(SqlJson(raw_data))
        .execute(&mut *tx)
        .await?;
    }

    sqlx::query(
        "UPDATE import_jobs
         SET import_status = 'mapping',
             total_rows = $2,
             processed_rows = 0,
             error_count = 0,
             updated_by = $3,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(total_rows)
    .bind(current.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let rows = sqlx::query_as::<_, ImportStagingRow>(
        "SELECT id, import_job_id, row_number, raw_data, normalized_data,
                row_status, error_count, created_at, updated_at
         FROM import_rows
         WHERE import_job_id = $1
         ORDER BY row_number ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn validate_import_job(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<ImportValidationSummary>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation"])?;
    let job = find_import_job(&state.pool, id).await?;

    let rows = sqlx::query_as::<_, ImportStagingRow>(
        "SELECT id, import_job_id, row_number, raw_data, normalized_data,
                row_status, error_count, created_at, updated_at
         FROM import_rows
         WHERE import_job_id = $1
         ORDER BY row_number ASC",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await?;

    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "import job has no staged rows".to_string(),
        ));
    }

    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM import_errors WHERE import_job_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    let mut valid_rows = 0;
    let mut invalid_rows = 0;
    let mut error_count = 0;

    for row in rows {
        let validation_errors = validate_row_for_module(&job.target_module, &row.raw_data);
        if validation_errors.is_empty() {
            let normalized = normalize_row(&row.raw_data);
            sqlx::query(
                "UPDATE import_rows
                 SET normalized_data = $3,
                     row_status = 'valid',
                     error_count = 0,
                     updated_at = now()
                 WHERE import_job_id = $1 AND row_number = $2",
            )
            .bind(id)
            .bind(row.row_number)
            .bind(SqlJson(normalized))
            .execute(&mut *tx)
            .await?;
            valid_rows += 1;
        } else {
            let row_error_count = validation_errors.len() as i32;
            for error in validation_errors {
                sqlx::query(
                    "INSERT INTO import_errors
                     (import_job_id, row_number, field_name, error_message, raw_data)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(id)
                .bind(row.row_number)
                .bind(error.field_name)
                .bind(error.message)
                .bind(SqlJson(row.raw_data.clone()))
                .execute(&mut *tx)
                .await?;
            }

            sqlx::query(
                "UPDATE import_rows
                 SET normalized_data = NULL,
                     row_status = 'invalid',
                     error_count = $3,
                     updated_at = now()
                 WHERE import_job_id = $1 AND row_number = $2",
            )
            .bind(id)
            .bind(row.row_number)
            .bind(row_error_count)
            .execute(&mut *tx)
            .await?;
            invalid_rows += 1;
            error_count += row_error_count;
        }
    }

    let next_status = if invalid_rows == 0 {
        "ready"
    } else {
        "validating"
    };
    sqlx::query(
        "UPDATE import_jobs
         SET import_status = $2,
             total_rows = $3,
             processed_rows = $4,
             error_count = $5,
             updated_by = $6,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(next_status)
    .bind(valid_rows + invalid_rows)
    .bind(valid_rows)
    .bind(error_count)
    .bind(current.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(ImportValidationSummary {
        import_job_id: id,
        total_rows: valid_rows + invalid_rows,
        valid_rows,
        invalid_rows,
        error_count,
    }))
}

async fn find_import_job(pool: &sqlx::PgPool, id: i64) -> ApiResult<ImportJobRow> {
    sqlx::query_as::<_, ImportJobRow>(
        "SELECT id, target_module, source_file_id, column_mapping, import_status,
                total_rows, processed_rows, error_count, note, created_at, updated_at, created_by
         FROM import_jobs
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

fn validate_target_module(module: &str) -> ApiResult<()> {
    match module {
        "vehicles" | "users" | "km_logs" | "maintenances" | "insurance_policies" | "expenses"
        | "damages" => Ok(()),
        _ => Err(ApiError::BadRequest(
            "invalid import target module".to_string(),
        )),
    }
}

fn validate_import_status(status: &str) -> ApiResult<()> {
    match status {
        "pending" | "mapping" | "validating" | "ready" | "importing" | "completed" | "failed"
        | "rolled_back" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid import status".to_string())),
    }
}

fn validate_row_for_module(module: &str, raw_data: &Value) -> Vec<RowValidationError> {
    let mut errors = Vec::new();
    if !raw_data.is_object() {
        errors.push(RowValidationError {
            field_name: None,
            message: "row must be a JSON object".to_string(),
        });
        return errors;
    }

    match module {
        "vehicles" => {
            require_text(raw_data, "plate", &mut errors);
            require_text(raw_data, "brand", &mut errors);
            require_text(raw_data, "model", &mut errors);
            optional_i64(raw_data, "company_id", &mut errors);
            optional_i64(raw_data, "department_id", &mut errors);
            optional_i64(raw_data, "user_id", &mut errors);
            optional_i32(raw_data, "model_year", &mut errors);
        }
        "users" => {
            require_text(raw_data, "email", &mut errors);
            require_text(raw_data, "full_name", &mut errors);
            require_allowed(
                raw_data,
                "role",
                &["admin", "manager", "operation", "accounting", "user"],
                &mut errors,
            );
            optional_i64(raw_data, "company_id", &mut errors);
            optional_i64(raw_data, "department_id", &mut errors);
        }
        "km_logs" => {
            require_i64(raw_data, "vehicle_id", &mut errors);
            require_i32(raw_data, "km", &mut errors);
            require_allowed(
                raw_data,
                "entry_type",
                &["manual", "ocr", "mobiliz_api", "kopilot_api"],
                &mut errors,
            );
        }
        "maintenances" => {
            require_i64(raw_data, "vehicle_id", &mut errors);
            optional_i32(raw_data, "last_maintenance_km", &mut errors);
            optional_i32(raw_data, "next_maintenance_km", &mut errors);
            optional_decimal(raw_data, "total_cost", &mut errors);
            optional_allowed(
                raw_data,
                "maintenance_status",
                &["planned", "scheduled", "completed", "cancelled"],
                &mut errors,
            );
        }
        "insurance_policies" => {
            require_i64(raw_data, "vehicle_id", &mut errors);
            require_allowed(
                raw_data,
                "policy_type",
                &["trafik", "kasko", "imm", "ferdi_kaza"],
                &mut errors,
            );
            require_text(raw_data, "policy_number", &mut errors);
            optional_decimal(raw_data, "amount", &mut errors);
            optional_allowed(
                raw_data,
                "renewal_status",
                &["active", "approaching", "renewing", "ended"],
                &mut errors,
            );
        }
        "expenses" => {
            require_i64(raw_data, "vehicle_id", &mut errors);
            require_allowed(
                raw_data,
                "expense_type",
                &[
                    "fuel",
                    "insurance",
                    "maintenance",
                    "tax",
                    "fine",
                    "tire",
                    "service",
                ],
                &mut errors,
            );
            require_decimal(raw_data, "amount", &mut errors);
            require_text(raw_data, "expense_date", &mut errors);
            optional_allowed(
                raw_data,
                "payment_status",
                &["pending", "paid", "cancelled", "overdue"],
                &mut errors,
            );
        }
        "damages" => {
            require_i64(raw_data, "vehicle_id", &mut errors);
            require_text(raw_data, "damage_date", &mut errors);
            optional_decimal(raw_data, "estimated_cost", &mut errors);
            optional_decimal(raw_data, "actual_cost", &mut errors);
            optional_allowed(
                raw_data,
                "damage_status",
                &[
                    "open",
                    "expertise",
                    "insurance",
                    "repaired",
                    "closed",
                    "cancelled",
                ],
                &mut errors,
            );
        }
        _ => errors.push(RowValidationError {
            field_name: None,
            message: "unsupported import target module".to_string(),
        }),
    }

    errors
}

fn normalize_row(raw_data: &Value) -> Value {
    match raw_data.as_object() {
        Some(object) => {
            let normalized = object
                .iter()
                .map(|(key, value)| (key.trim().to_lowercase(), value.clone()))
                .collect::<serde_json::Map<String, Value>>();
            Value::Object(normalized)
        }
        None => raw_data.clone(),
    }
}

fn require_text(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    match raw_data.get(field).and_then(Value::as_str) {
        Some(value) if !value.trim().is_empty() => {}
        _ => push_field_error(field, "field is required", errors),
    }
}

fn require_i64(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    if raw_data.get(field).and_then(value_to_i64).is_none() {
        push_field_error(field, "field must be an integer", errors);
    }
}

fn require_i32(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    match raw_data.get(field).and_then(value_to_i64) {
        Some(value) if i32::try_from(value).is_ok() => {}
        _ => push_field_error(field, "field must be a 32-bit integer", errors),
    }
}

fn optional_i64(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    if raw_data
        .get(field)
        .filter(|value| !value.is_null())
        .is_some()
        && raw_data.get(field).and_then(value_to_i64).is_none()
    {
        push_field_error(field, "field must be an integer", errors);
    }
}

fn optional_i32(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    if let Some(value) = raw_data.get(field).filter(|value| !value.is_null()) {
        match value_to_i64(value) {
            Some(number) if i32::try_from(number).is_ok() => {}
            _ => push_field_error(field, "field must be a 32-bit integer", errors),
        }
    }
}

fn require_decimal(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    if raw_data
        .get(field)
        .and_then(value_to_decimal_string)
        .is_none()
    {
        push_field_error(field, "field must be a decimal number", errors);
    }
}

fn optional_decimal(raw_data: &Value, field: &str, errors: &mut Vec<RowValidationError>) {
    if raw_data
        .get(field)
        .filter(|value| !value.is_null())
        .is_some()
        && raw_data
            .get(field)
            .and_then(value_to_decimal_string)
            .is_none()
    {
        push_field_error(field, "field must be a decimal number", errors);
    }
}

fn require_allowed(
    raw_data: &Value,
    field: &str,
    allowed: &[&str],
    errors: &mut Vec<RowValidationError>,
) {
    match raw_data.get(field).and_then(Value::as_str) {
        Some(value) if allowed.contains(&value) => {}
        Some(_) => push_field_error(field, "field has an invalid value", errors),
        None => push_field_error(field, "field is required", errors),
    }
}

fn optional_allowed(
    raw_data: &Value,
    field: &str,
    allowed: &[&str],
    errors: &mut Vec<RowValidationError>,
) {
    if let Some(value) = raw_data.get(field).filter(|value| !value.is_null()) {
        match value.as_str() {
            Some(text) if allowed.contains(&text) => {}
            _ => push_field_error(field, "field has an invalid value", errors),
        }
    }
}

fn value_to_i64(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        value
            .as_str()
            .and_then(|text| text.trim().parse::<i64>().ok())
    })
}

fn value_to_decimal_string(value: &Value) -> Option<String> {
    if value.is_number() {
        Some(value.to_string())
    } else {
        value
            .as_str()
            .filter(|text| text.trim().parse::<rust_decimal::Decimal>().is_ok())
            .map(|text| text.trim().to_string())
    }
}

fn push_field_error(field: &str, message: &str, errors: &mut Vec<RowValidationError>) {
    errors.push(RowValidationError {
        field_name: Some(field.to_string()),
        message: message.to_string(),
    });
}
