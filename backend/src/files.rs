use crate::{
    audit, auth,
    error::{ApiError, ApiResult},
    AppState,
};
use axum::{
    extract::{Multipart, Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use sanitize_filename::sanitize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{types::Json as SqlJson, FromRow, PgPool};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_files))
        .route("/upload", post(upload_file))
        .route("/:id", get(get_file).delete(delete_file))
}

#[derive(Debug, Deserialize)]
struct FileQuery {
    module_name: Option<String>,
    entity_id: Option<i64>,
    file_type: Option<String>,
    limit: Option<i64>,
}

#[derive(Debug, FromRow, Serialize)]
struct FileDocumentRow {
    id: i64,
    module_name: String,
    entity_id: i64,
    file_type: String,
    original_name: String,
    storage_path: String,
    mime_type: Option<String>,
    file_size: i64,
    checksum_sha256: Option<String>,
    note: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    created_by: Option<i64>,
}

#[derive(Debug)]
struct UploadPayload {
    module_name: String,
    entity_id: i64,
    file_type: String,
    note: Option<String>,
    original_name: String,
    mime_type: Option<String>,
    bytes: Vec<u8>,
}

async fn list_files(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<FileQuery>,
) -> ApiResult<Json<Vec<FileDocumentRow>>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;

    let rows = sqlx::query_as::<_, FileDocumentRow>(
        "SELECT id, module_name, entity_id, file_type, original_name, storage_path,
                mime_type, file_size, checksum_sha256, note, created_at, updated_at, created_by
         FROM file_documents
         WHERE deleted_at IS NULL
           AND ($1::text IS NULL OR module_name = $1)
           AND ($2::bigint IS NULL OR entity_id = $2)
           AND ($3::text IS NULL OR file_type = $3)
         ORDER BY created_at DESC
         LIMIT $4",
    )
    .bind(query.module_name)
    .bind(query.entity_id)
    .bind(query.file_type)
    .bind(query.limit.unwrap_or(100).clamp(1, 500))
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    multipart: Multipart,
) -> ApiResult<Json<FileDocumentRow>> {
    let current = auth::current_user(&state, &headers).await?;

    let payload = read_upload_payload(multipart).await?;
    validate_module_name(&payload.module_name)?;
    validate_file_type(&payload.file_type)?;
    validate_file_type_for_module(&payload.module_name, &payload.file_type)?;
    if payload.module_name == "support_tickets" {
        auth::require_roles_or_mobile_permission(
            &state.pool,
            &current,
            &["admin", "manager", "operation", "accounting"],
            "media_upload",
            "create",
        )
        .await?;
    } else if payload.module_name == "vehicles" && payload.file_type == "km_photo" {
        auth::require_roles_or_mobile_permission(
            &state.pool,
            &current,
            &["admin", "manager", "operation", "accounting"],
            "km_log",
            "create",
        )
        .await?;
    } else {
        auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    }
    ensure_entity_exists(&state.pool, &payload.module_name, payload.entity_id).await?;

    let checksum = hex::encode(Sha256::digest(&payload.bytes));
    let stored_path = write_file_to_storage(&state, &payload, &checksum).await?;
    let storage_path = stored_path.to_string_lossy().replace('\\', "/");

    let row = sqlx::query_as::<_, FileDocumentRow>(
        "INSERT INTO file_documents
         (module_name, entity_id, file_type, original_name, storage_path,
          mime_type, file_size, checksum_sha256, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING id, module_name, entity_id, file_type, original_name, storage_path,
                   mime_type, file_size, checksum_sha256, note, created_at, updated_at, created_by",
    )
    .bind(payload.module_name)
    .bind(payload.entity_id)
    .bind(payload.file_type)
    .bind(payload.original_name)
    .bind(storage_path)
    .bind(payload.mime_type)
    .bind(payload.bytes.len() as i64)
    .bind(checksum)
    .bind(payload.note)
    .bind(current.id)
    .fetch_one(&state.pool)
    .await?;

    create_media_ai_job_if_needed(&state.pool, &row, current.id).await?;

    audit::write_audit(
        &state.pool,
        "file_documents",
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

async fn get_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<FileDocumentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager", "operation", "accounting"])?;
    Ok(Json(find_file(&state.pool, id).await?))
}

async fn delete_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> ApiResult<Json<FileDocumentRow>> {
    let current = auth::current_user(&state, &headers).await?;
    auth::require_roles(&current, &["admin", "manager"])?;
    let old = find_file(&state.pool, id).await?;

    let row = sqlx::query_as::<_, FileDocumentRow>(
        "UPDATE file_documents
         SET deleted_at = now(),
             is_active = false,
             updated_by = $2,
             updated_at = now()
         WHERE id = $1 AND deleted_at IS NULL
         RETURNING id, module_name, entity_id, file_type, original_name, storage_path,
                   mime_type, file_size, checksum_sha256, note, created_at, updated_at, created_by",
    )
    .bind(id)
    .bind(current.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    audit::write_audit(
        &state.pool,
        "file_documents",
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

async fn read_upload_payload(mut multipart: Multipart) -> ApiResult<UploadPayload> {
    let mut module_name: Option<String> = None;
    let mut entity_id: Option<i64> = None;
    let mut file_type: Option<String> = None;
    let mut note: Option<String> = None;
    let mut original_name: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| ApiError::BadRequest(err.to_string()))?
    {
        let name = field.name().unwrap_or_default().to_string();
        match name.as_str() {
            "module_name" => module_name = Some(read_text_field(field).await?),
            "entity_id" => {
                let value = read_text_field(field).await?;
                entity_id =
                    Some(value.parse().map_err(|_| {
                        ApiError::BadRequest("entity_id must be a number".to_string())
                    })?);
            }
            "file_type" => file_type = Some(read_text_field(field).await?),
            "note" => note = Some(read_text_field(field).await?),
            "file" => {
                original_name = field.file_name().map(str::to_string);
                mime_type = field.content_type().map(str::to_string);
                bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|err| ApiError::BadRequest(err.to_string()))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    Ok(UploadPayload {
        module_name: module_name
            .ok_or_else(|| ApiError::BadRequest("module_name is required".to_string()))?,
        entity_id: entity_id
            .ok_or_else(|| ApiError::BadRequest("entity_id is required".to_string()))?,
        file_type: file_type
            .ok_or_else(|| ApiError::BadRequest("file_type is required".to_string()))?,
        note,
        original_name: original_name
            .ok_or_else(|| ApiError::BadRequest("file is required".to_string()))?,
        mime_type,
        bytes: bytes.ok_or_else(|| ApiError::BadRequest("file is required".to_string()))?,
    })
}

async fn read_text_field(field: axum::extract::multipart::Field<'_>) -> ApiResult<String> {
    field
        .text()
        .await
        .map_err(|err| ApiError::BadRequest(err.to_string()))
}

async fn write_file_to_storage(
    state: &AppState,
    payload: &UploadPayload,
    checksum: &str,
) -> ApiResult<PathBuf> {
    let safe_module = sanitize(&payload.module_name);
    let safe_type = sanitize(&payload.file_type);
    let safe_name = sanitize(&payload.original_name);
    let short_checksum = checksum.get(0..16).unwrap_or(checksum);
    let file_name = format!("{}_{}", short_checksum, safe_name);
    let relative_path = PathBuf::from(safe_module)
        .join(payload.entity_id.to_string())
        .join(safe_type)
        .join(file_name);
    let full_path = state.config.storage_root.join(&relative_path);

    if let Some(parent) = full_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|err| ApiError::Internal(err.into()))?;
    }

    let mut file = tokio::fs::File::create(&full_path)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;
    file.write_all(&payload.bytes)
        .await
        .map_err(|err| ApiError::Internal(err.into()))?;

    Ok(relative_path)
}

async fn find_file(pool: &sqlx::PgPool, id: i64) -> ApiResult<FileDocumentRow> {
    sqlx::query_as::<_, FileDocumentRow>(
        "SELECT id, module_name, entity_id, file_type, original_name, storage_path,
                mime_type, file_size, checksum_sha256, note, created_at, updated_at, created_by
         FROM file_documents
         WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound)
}

async fn create_media_ai_job_if_needed(
    pool: &PgPool,
    row: &FileDocumentRow,
    actor_id: i64,
) -> ApiResult<()> {
    let Some(analysis_type) = media_analysis_type(&row.module_name, &row.file_type) else {
        return Ok(());
    };

    let related_vehicle_id = related_vehicle_for_media(pool, row).await?;
    let input_data = json!({
        "source": "file_upload",
        "file_document": {
            "id": row.id,
            "module_name": row.module_name,
            "entity_id": row.entity_id,
            "file_type": row.file_type,
            "original_name": row.original_name,
            "storage_path": row.storage_path,
            "mime_type": row.mime_type,
            "file_size": row.file_size,
            "note": row.note,
            "created_at": row.created_at
        },
        "media_contract": {
            "task": analysis_type,
            "supports_local_vision_model": true,
            "supports_external_vision_model": true,
            "do_not_modify_source_record": true,
            "do_not_send_customer_message": true,
            "human_review_required": true,
            "expected_output": [
                "summary",
                "detected_issue",
                "severity_suggestion",
                "evidence",
                "recommended_actions",
                "missing_information"
            ]
        }
    });

    sqlx::query(
        "INSERT INTO ai_analysis_jobs
         (analysis_type, related_vehicle_id, source_table, source_record_id,
          input_data, requires_human_approval, created_by)
         VALUES ($1, $2, 'file_documents', $3, $4, true, $5)",
    )
    .bind(analysis_type)
    .bind(related_vehicle_id)
    .bind(row.id)
    .bind(SqlJson(input_data))
    .bind(actor_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn media_analysis_type(module_name: &str, file_type: &str) -> Option<&'static str> {
    match (module_name, file_type) {
        ("vehicles", "vehicle_photo") => Some("vehicle_media_inspection"),
        ("damages", "damage_photo") => Some("vehicle_media_inspection"),
        (
            "support_tickets",
            "support_screenshot" | "support_video" | "support_log" | "support_document",
        ) => Some("support_media_analysis"),
        _ => None,
    }
}

async fn related_vehicle_for_media(pool: &PgPool, row: &FileDocumentRow) -> ApiResult<Option<i64>> {
    match row.module_name.as_str() {
        "vehicles" => Ok(Some(row.entity_id)),
        "damages" => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT vehicle_id FROM damages WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(row.entity_id)
        .fetch_optional(pool)
        .await?),
        _ => Ok(None),
    }
}

fn validate_module_name(module_name: &str) -> ApiResult<()> {
    match module_name {
        "vehicles"
        | "insurance_policies"
        | "expenses"
        | "maintenances"
        | "damages"
        | "sold_vehicles"
        | "inventory_products"
        | "support_tickets"
        | "support_knowledge_base" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid file module".to_string())),
    }
}

fn validate_file_type(file_type: &str) -> ApiResult<()> {
    match file_type {
        "registration"
        | "delivery_form"
        | "vehicle_photo"
        | "km_photo"
        | "policy_pdf"
        | "offer"
        | "invoice"
        | "payment_receipt"
        | "expense_document"
        | "damage_report"
        | "expert_report"
        | "damage_photo"
        | "sale_document"
        | "transfer_document"
        | "technical_document"
        | "warranty_document"
        | "product_photo"
        | "usage_instruction"
        | "support_screenshot"
        | "support_video"
        | "support_log"
        | "support_document"
        | "knowledge_attachment" => Ok(()),
        _ => Err(ApiError::BadRequest("invalid file type".to_string())),
    }
}

fn validate_file_type_for_module(module_name: &str, file_type: &str) -> ApiResult<()> {
    let allowed = match module_name {
        "vehicles" => matches!(
            file_type,
            "registration" | "delivery_form" | "vehicle_photo" | "km_photo"
        ),
        "insurance_policies" => matches!(
            file_type,
            "policy_pdf" | "offer" | "invoice" | "payment_receipt"
        ),
        "expenses" => matches!(
            file_type,
            "invoice" | "payment_receipt" | "expense_document"
        ),
        "maintenances" => matches!(
            file_type,
            "invoice" | "payment_receipt" | "expense_document"
        ),
        "damages" => matches!(
            file_type,
            "damage_report" | "expert_report" | "damage_photo" | "invoice"
        ),
        "sold_vehicles" => matches!(file_type, "sale_document" | "transfer_document"),
        "inventory_products" => matches!(
            file_type,
            "invoice"
                | "technical_document"
                | "warranty_document"
                | "product_photo"
                | "usage_instruction"
        ),
        "support_tickets" => matches!(
            file_type,
            "support_screenshot" | "support_video" | "support_log" | "support_document"
        ),
        "support_knowledge_base" => matches!(
            file_type,
            "knowledge_attachment" | "technical_document" | "usage_instruction"
        ),
        _ => false,
    };

    if allowed {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "file type is not allowed for this module".to_string(),
        ))
    }
}

async fn ensure_entity_exists(pool: &PgPool, module_name: &str, entity_id: i64) -> ApiResult<()> {
    let exists = match module_name {
        "vehicles" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM vehicles WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "insurance_policies" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM insurance_policies WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "expenses" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM expenses WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "maintenances" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM maintenances WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "damages" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM damages WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "sold_vehicles" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM sold_vehicles WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "inventory_products" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM inventory_products WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "support_tickets" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM support_tickets WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        "support_knowledge_base" => exists_by_id(pool, "SELECT EXISTS(SELECT 1 FROM support_knowledge_base WHERE id = $1 AND deleted_at IS NULL)", entity_id).await?,
        _ => false,
    };

    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

async fn exists_by_id(pool: &PgPool, sql: &str, entity_id: i64) -> ApiResult<bool> {
    Ok(sqlx::query_scalar::<_, bool>(sql)
        .bind(entity_id)
        .fetch_one(pool)
        .await?)
}
